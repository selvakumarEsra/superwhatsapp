---
layout: home
title: SuperWhatsapp
description: WhatsApp on your desktop with Znga, a proactive AI agent that tells you what needs you, briefs you on what you missed and drafts replies in your voice. Fast, private, for macOS, Windows and Linux.
permalink: /
hero:
  name: SuperWhatsapp
  text: Your WhatsApp, with an AI agent that works ahead of you
  tagline: SuperWhatsapp is a fast desktop app for WhatsApp with Znga built in. Znga spots the messages that need you, briefs you on what you missed and drafts replies in your voice. Nothing is sent without your click.
  actions:
    - theme: brand
      text: Download free
      link: /download/
    - theme: alt
      text: Meet Znga, the AI agent
      link: /znga/
    - theme: alt
      text: See how it works
      link: /what-is-superwhatsapp/
  image:
    dark: /screenshot.png
    light: /screenshot-light.png
    alt: "SuperWhatsapp with Znga's Today panel beside a chat: a message that needs a reply, with buttons to open the chat or draft a reply, and a briefing of what happened while you were away"
    width: 1800
    height: 1157

features:
  - icon: ✨
    title: An agent that acts first
    details: Znga reads new messages as they arrive and raises the ones that need you (a question, a deadline, a friend writing again and again) before they get buried.
    link: /znga/#needs-you
    link_text: How Znga decides
  - icon: 📰
    title: Briefings, not backlogs
    details: Back from a meeting or a flight? One click gives you what happened and what needs you, across every chat, in a few lines.
    link: /znga/#briefings
    link_text: See a briefing
  - icon: ✍️
    title: Replies in your voice
    details: Znga learns how you write and drafts replies you can send, edit or throw away. "Tell Mom and Alex I'm 20 minutes late" becomes one personal message each.
    link: /znga/#drafts
    link_text: Drafts and tone tools
  - icon: 🧠
    title: Memory that keeps promises
    details: Znga remembers what you promised and reminds you before it is due, plus the facts people tell you, encrypted on your computer.
    link: /znga/#memory
    link_text: What Znga remembers
  - icon: 🔒
    title: Private by design
    details: Your archive is encrypted on your computer. Znga is off until you turn it on, never reads locked chats, and can run fully offline with a local model.
    link: /privacy/
    link_text: Privacy and security
  - icon: ⚡
    title: Native and fast
    details: Written in Rust, with no browser inside. It starts quickly, scrolls smoothly through years of history and stays light on memory and battery.
    link: /how-it-links/
    link_text: How it connects
  - icon: 📊
    title: Insights about your chats
    details: Your busiest hours, reply times, streaks and the people you have drifted from, worked out on your computer. Nothing leaves it.
    link: /using-superwhatsapp/
    link_text: Explore Insights
  - icon: 💬
    title: Everything you expect
    details: Voice messages, photos, stickers, polls, calls (beta), several numbers, favourites, archive, starred messages, themes, an app lock and keyboard shortcuts.
    link: /using-superwhatsapp/
    link_text: The full tour
  - icon: 🖥️
    title: Mac, Windows and Linux
    details: One app on every desktop, with the tray, notifications and badges each one expects. Free during the beta, open source under the MIT license.
    link: /download/
    link_text: Download
---

<div class="sw-section">
  <h2>Stop scrolling to find what matters</h2>
  <p>Most messaging time goes on working out which messages need you. Znga does that part. It scores every new message on your computer, without sending it anywhere, and keeps a short list of what needs you today. When you have been away it writes a briefing. When you want to reply it drafts one in your voice. You stay in charge of every word that leaves your account.</p>
</div>

<div class="sw-shots">
  <figure>
    <img src="{{ '/assets/images/product/znga-drafts.png' | relative_url }}" alt="Znga's Drafts tab: reply suggestions, tone tools, and two personal 'running late' updates waiting for Send" loading="lazy">
    <figcaption><strong>Drafts.</strong> Suggest a reply, rewrite it friendlier, shorter or more formal, translate it, or send one personal update to several people. Each draft waits for your click.</figcaption>
  </figure>
  <figure>
    <img src="{{ '/assets/images/product/insights.png' | relative_url }}" alt="Insights overview: messages sent and received, active chats, streak and highlights such as reply times" loading="lazy">
    <figcaption><strong>Insights.</strong> How you use WhatsApp, at a glance: trends, rhythm, reply times and the friends you have not talked to in a while.</figcaption>
  </figure>
</div>

<div class="sw-section">
  <h2>Built for people who live in WhatsApp</h2>
  <div class="sw-grid">
    <div><h3>Founders and freelancers</h3><p>Clients, suppliers and team chats in one place. Znga flags the client who asked a question this morning and drafts the follow-up.</p></div>
    <div><h3>Busy parents</h3><p>School groups, family plans and the friend who needs an answer. Get the two things that matter out of three hundred messages.</p></div>
    <div><h3>Teams and support</h3><p>Several numbers linked side by side, keyboard-first, with an app lock and an encrypted archive on your own machine.</p></div>
  </div>
</div>

<div class="sw-section">
  <h2>Your AI, your choice</h2>
  <p>Znga works with the AI you already trust: Claude (through Claude Code or an API key), Gemini, any OpenAI-compatible service, or a model running entirely on your computer with Ollama or LM Studio. Pick a local model and nothing you read or write ever leaves your machine.</p>
  <p><a class="sw-button" href="{{ '/download/' | relative_url }}">Download SuperWhatsapp</a> <a class="sw-link" href="{{ '/faq/' | relative_url }}">Questions? Read the FAQ</a></p>
</div>

<p class="sw-note">SuperWhatsapp is an independent app that links to your WhatsApp account as a companion device, the way WhatsApp's own desktop app does. It is not made by, affiliated with or endorsed by WhatsApp or Meta. Using an unofficial client may be against WhatsApp's terms of service.</p>

<style>
  .sw-section { max-width: 1152px; margin: 64px auto 0; padding: 0 24px; }
  .sw-section h2 { font-size: 28px; line-height: 1.25; font-weight: 700; margin-bottom: 12px; border: 0; padding: 0; }
  .sw-section h3 { font-size: 17px; font-weight: 600; margin: 0 0 6px; }
  .sw-section p { color: var(--vp-c-text-2); line-height: 1.7; max-width: 760px; }
  .sw-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; margin-top: 16px; }
  .sw-grid > div { background: var(--vp-c-bg-soft); border-radius: 12px; padding: 20px; }
  .sw-grid p { margin: 0; }
  .sw-shots { max-width: 1152px; margin: 40px auto 0; padding: 0 24px; display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 24px; }
  .sw-shots figure { margin: 0; }
  .sw-shots img { width: 100%; height: auto; border-radius: 12px; box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35); }
  .sw-shots figcaption { margin-top: 12px; color: var(--vp-c-text-2); line-height: 1.6; font-size: 14px; }
  .sw-button { display: inline-block; padding: 10px 20px; border-radius: 20px; background: var(--vp-c-brand-1); color: var(--vp-c-white) !important; font-weight: 600; text-decoration: none !important; margin-right: 16px; }
  .sw-link { font-weight: 500; }
  .sw-note { max-width: 1152px; margin: 56px auto 0; padding: 0 24px; text-align: center; color: var(--vp-c-text-3); font-size: 13px; line-height: 1.6; }
  .VPHero .image-container { width: 100% !important; height: auto !important; transform: none !important; }
  .VPHero .image-src {
    position: relative !important; top: auto !important; left: auto !important; transform: none !important;
    width: 100% !important; height: auto !important; max-width: 100% !important; max-height: none !important;
    padding: 0 !important; border-radius: 12px; box-shadow: 0 12px 48px rgba(0, 0, 0, 0.45);
  }
  @media (max-width: 959px) { .VPHero .image { margin: 0 0 24px !important; } }
</style>
