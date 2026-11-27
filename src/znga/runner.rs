//! Znga's worker threads: model calls never run on the interface thread.
//!
//! Two lanes (REQ-ZNGA-015): interactive requests (Ask, the person's own
//! chat, rewrites, explanations…) and background ones (briefings,
//! extraction, automatic drafts) each have their own thread, so a slow
//! briefing never holds up a question. Answers stream: partial text arrives
//! as it is written, and a request can be stopped.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

use super::provider::{self, Prompt, Target};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lane {
    Interactive,
    Background,
}

struct Request {
    id: u64,
    target: Target,
    prompt: Prompt,
    cancel: Arc<AtomicBool>,
}

enum Message {
    Partial(u64, String),
    Done(u64, Result<String, String>),
}

#[cfg(test)]
type Fake = Box<dyn Fn(&Prompt) -> Result<String, String>>;

struct Worker {
    sender: Sender<Request>,
}

struct Pending<P> {
    purpose: P,
    lane: Lane,
    cancel: Arc<AtomicBool>,
}

pub struct Runner<P> {
    workers: HashMap<Lane, Worker>,
    outbox: Sender<Message>,
    answers: Receiver<Message>,
    waker: crate::backend::Waker,
    next: u64,
    pending: HashMap<u64, Pending<P>>,
    /// Text written so far, per running request.
    partial: HashMap<u64, String>,
    /// Test hook: answers requests in place of the network.
    #[cfg(test)]
    pub fake: Option<Fake>,
    #[cfg(test)]
    fake_answers: Vec<(u64, Result<String, String>)>,
    /// Test hook: where the last request would have gone.
    #[cfg(test)]
    pub last_target: Option<Target>,
}

/// More queued background requests than this and new optional ones are
/// dropped.
pub const BUSY: usize = 6;

/// What a stopped request ends with.
pub const STOPPED: &str = "Stopped.";

impl<P> Runner<P> {
    pub fn new(waker: crate::backend::Waker) -> Self {
        let (outbox, answers) = channel();
        Self {
            workers: HashMap::new(),
            outbox,
            answers,
            waker,
            next: 0,
            pending: HashMap::new(),
            partial: HashMap::new(),
            #[cfg(test)]
            fake: None,
            #[cfg(test)]
            fake_answers: Vec::new(),
            #[cfg(test)]
            last_target: None,
        }
    }

    pub fn busy(&self) -> usize {
        self.pending.len()
    }

    /// Requests waiting or running on `lane`.
    pub fn busy_on(&self, lane: Lane) -> usize {
        self.pending.values().filter(|p| p.lane == lane).count()
    }

    pub fn is_waiting(&self, matches: impl Fn(&P) -> bool) -> bool {
        self.pending.values().any(|p| matches(&p.purpose))
    }

    /// The running request matching `matches` and the text it has written
    /// so far.
    pub fn partial(&self, matches: impl Fn(&P) -> bool) -> Option<(u64, &str)> {
        self.pending
            .iter()
            .filter(|(_, p)| matches(&p.purpose))
            .map(|(id, _)| (*id, self.partial.get(id).map_or("", String::as_str)))
            .min_by_key(|(id, _)| *id)
    }

    /// Asks request `id` to stop; it ends with [`STOPPED`] and its partial
    /// text.
    pub fn cancel(&mut self, id: u64) {
        if let Some(pending) = self.pending.get(&id) {
            pending.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Queues `prompt` on `lane`; its answer comes back from [`Self::poll`].
    pub fn submit(&mut self, target: Target, prompt: Prompt, purpose: P, lane: Lane) -> u64 {
        self.next += 1;
        let id = self.next;
        let cancel = Arc::new(AtomicBool::new(false));
        self.pending.insert(
            id,
            Pending {
                purpose,
                lane,
                cancel: Arc::clone(&cancel),
            },
        );
        #[cfg(test)]
        if let Some(fake) = &self.fake {
            self.last_target = Some(target.clone());
            let answer = fake(&prompt);
            self.fake_answers.push((id, answer));
            return id;
        }
        let mut request = Request {
            id,
            target,
            prompt,
            cancel,
        };
        for _ in 0..2 {
            if !self.workers.contains_key(&lane) {
                self.start(lane);
            }
            let Some(worker) = self.workers.get(&lane) else {
                break;
            };
            match worker.sender.send(request) {
                Ok(()) => return id,
                Err(returned) => {
                    // The thread is gone: start a new one and try once more.
                    request = returned.0;
                    self.workers.remove(&lane);
                }
            }
        }
        let _ = self
            .outbox
            .send(Message::Done(id, Err("Znga's worker stopped.".into())));
        id
    }

    fn start(&mut self, lane: Lane) {
        let (requests, inbox) = channel::<Request>();
        let outbox = self.outbox.clone();
        let waker = self.waker.clone();
        let name = match lane {
            Lane::Interactive => "znga-interactive",
            Lane::Background => "znga-background",
        };
        let spawned = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                for request in inbox {
                    let id = request.id;
                    let answer = {
                        let outbox = outbox.clone();
                        let waker = waker.clone();
                        let mut on_delta = move |delta: &str| {
                            let _ = outbox.send(Message::Partial(id, delta.to_owned()));
                            waker.wake();
                        };
                        provider::complete_streaming(
                            &request.target,
                            &request.prompt,
                            &mut on_delta,
                            &request.cancel,
                        )
                    };
                    if outbox.send(Message::Done(id, answer)).is_err() {
                        break;
                    }
                    waker.wake();
                }
            });
        match spawned {
            Ok(_) => {
                self.workers.insert(lane, Worker { sender: requests });
            }
            Err(_) => log::warn!("znga: could not start its worker thread"),
        }
    }

    /// Answers that finished, with their purposes. A stopped request ends
    /// with its partial text followed by [`STOPPED`].
    pub fn poll(&mut self) -> Vec<(P, Result<String, String>)> {
        let mut out = Vec::new();
        #[cfg(test)]
        for (id, answer) in std::mem::take(&mut self.fake_answers) {
            if let Some(pending) = self.pending.remove(&id) {
                out.push((pending.purpose, answer));
            }
        }
        loop {
            match self.answers.try_recv() {
                Ok(Message::Partial(id, delta)) => {
                    if self.pending.contains_key(&id) {
                        self.partial.entry(id).or_default().push_str(&delta);
                    }
                }
                Ok(Message::Done(id, answer)) => {
                    let written = self.partial.remove(&id).unwrap_or_default();
                    if let Some(pending) = self.pending.remove(&id) {
                        let answer = match answer {
                            Err(error) if error == STOPPED => {
                                let written = written.trim();
                                if written.is_empty() {
                                    Err(STOPPED.to_owned())
                                } else {
                                    Ok(format!("{written}\n\n({STOPPED})"))
                                }
                            }
                            other => other,
                        };
                        out.push((pending.purpose, answer));
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        out
    }
}

/// Which lane a request belongs on.
pub fn lane_for(background: bool) -> Lane {
    if background {
        Lane::Background
    } else {
        Lane::Interactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_text_accumulates_and_a_stop_keeps_it() {
        let mut runner: Runner<&str> = Runner::new(crate::backend::Waker::default());
        runner.pending.insert(
            7,
            Pending {
                purpose: "ask",
                lane: Lane::Interactive,
                cancel: Arc::new(AtomicBool::new(false)),
            },
        );
        runner
            .outbox
            .send(Message::Partial(7, "Hel".into()))
            .unwrap();
        runner
            .outbox
            .send(Message::Partial(7, "lo".into()))
            .unwrap();
        assert!(runner.poll().is_empty());
        assert_eq!(runner.partial(|p| *p == "ask"), Some((7, "Hello")));
        runner.cancel(7);
        assert!(runner.pending[&7].cancel.load(Ordering::Relaxed));
        runner
            .outbox
            .send(Message::Done(7, Err(STOPPED.into())))
            .unwrap();
        let done = runner.poll();
        assert_eq!(done, vec![("ask", Ok("Hello\n\n(Stopped.)".to_owned()))]);
        assert!(runner.partial(|_| true).is_none());
    }

    #[test]
    fn lanes_are_counted_apart() {
        let mut runner: Runner<&str> = Runner::new(crate::backend::Waker::default());
        runner.fake = Some(Box::new(|_| Ok("x".into())));
        let target = provider::tests_support::target();
        let prompt = provider::tests_support::prompt();
        runner.submit(target.clone(), prompt.clone(), "brief", Lane::Background);
        runner.submit(target, prompt, "ask", Lane::Interactive);
        assert_eq!(runner.busy_on(Lane::Background), 1);
        assert_eq!(runner.busy_on(Lane::Interactive), 1);
    }
}
