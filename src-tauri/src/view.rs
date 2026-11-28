//! What the web interface shows, built from the core's state after each
//! step (REQ-WEBUI-002). The page never holds the state itself.

use serde::Serialize;
use zapfast::app::App;
use zapfast::backend::LinkStatus;
use zapfast::model::{Chat, ChatFilter, Content, Delivery, Message, Page};

/// The most messages of the open chat sent at once.
const WINDOW: usize = 300;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub link: Link,
    /// The current call, while one is up.
    pub call: Option<CallView>,
    pub me: Option<String>,
    /// Your own picture, for the rail.
    pub me_avatar: Option<String>,
    /// The filter chips over the list, with their unread counts.
    pub filters: Vec<FilterChip>,
    pub filter: &'static str,
    pub search: String,
    pub chats: Vec<ChatRow>,
    /// The Znga chat's row, kept above every other chat while Znga is on
    /// (REQ-ZCHAT-001).
    pub znga_row: Option<ChatRow>,
    /// The Znga chat, while it is the open conversation.
    pub znga: Option<crate::znga::ZngaChat>,
    /// Text Znga offers to a chat's composer (REQ-ZCHAT-004).
    pub offer: Option<crate::znga::OfferView>,
    /// Unread chats, muted and archived ones left out: the sidebar's count,
    /// like the Dock badge.
    pub unread_chats: u32,
    pub open: Option<Open>,
    /// What the sidebar shows as chosen: chats, favourites, archived,
    /// starred or calls.
    pub section: &'static str,
    /// The starred messages, newest first, while Starred is shown.
    pub starred: Vec<StarredRow>,
    /// The call log, newest first, while Calls is shown.
    pub calls: Vec<CallRow>,
    /// all or missed: the Calls page's chip.
    pub call_filter: &'static str,
    pub calls_search: String,
    /// The contact or group info panel, when open.
    pub info: Option<Info>,
    pub settings: SettingsView,
    /// The lock screen, while the app lock holds; nothing private is sent
    /// then (REQ-WEBUI-005).
    pub lock: Option<LockView>,
    /// The locked-chats folder behind the rail's Lock plate.
    pub locked: LockedView,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedView {
    /// The rail offers the plate: chats are locked, or the folder is open.
    pub offered: bool,
    pub open: bool,
    /// The code prompt, while it is up.
    pub prompt: Option<LockedPrompt>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedPrompt {
    /// No code yet: choose one, and confirm it.
    pub setup: bool,
    /// The last code did not match.
    pub error: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterChip {
    pub id: &'static str,
    pub label: String,
    /// Unread chats it holds; 0 for All.
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockView {
    /// The last password was wrong.
    pub wrong: bool,
    /// Seconds before another try, after several wrong ones.
    pub wait: Option<u64>,
    pub checking: bool,
    /// no, confirming or unlinking: where "Forgot password?" got to.
    pub forgetting: &'static str,
}

/// The app lock's Settings form while one is open.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockForm {
    /// set, change or turnOff.
    pub mode: &'static str,
    /// tooShort, mismatch or wrongCurrent.
    pub error: Option<&'static str>,
    pub busy: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Blocked {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Swatch {
    pub id: String,
    pub label: &'static str,
    /// `#rrggbb`, or `None` for the theme's own colour.
    pub color: Option<String>,
}

/// Every setting the Settings screen shows, with its stored value.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    /// dark, light or system.
    pub theme: &'static str,
    pub zoom: f32,
    pub show_wallpaper: bool,
    pub enter_sends: bool,
    /// The shortcut line under the composer (Keyboard shortcuts).
    pub show_shortcut_hints: bool,
    /// system or inter: Settings › Appearance › Font.
    pub font: &'static str,
    pub auto_download: bool,
    pub history_prefetch: bool,
    pub keep_chats_archived: bool,
    pub pause_other_media: bool,
    pub notifications: bool,
    pub notification_content: bool,
    pub group_sounds: bool,
    /// receive, alert, system, none or custom.
    pub message_sound: &'static str,
    pub mention_sound: &'static str,
    pub send_read_receipts: bool,
    pub send_typing: bool,
    pub keep_running: bool,
    /// `None` where the platform cannot start apps at login.
    pub start_with_system: Option<bool>,
    pub check_for_updates: bool,
    pub download_updates: bool,
    pub proxy: String,
    pub name: Option<String>,
    pub about: Option<String>,
    pub phone: Option<String>,
    pub version: &'static str,
    /// Settings › Files, as egui shows it.
    pub files: FilesView,
    /// The wallpaper colour chosen for the light and the dark theme.
    pub wallpaper_light: String,
    pub wallpaper_dark: String,
    pub swatches_light: Vec<Swatch>,
    pub swatches_dark: Vec<Swatch>,
    /// A secret code hides locked chats.
    pub chat_lock_code: bool,
    /// The app asks for a password.
    pub app_lock: bool,
    /// oneMinute, fifteenMinutes or oneHour.
    pub auto_lock: &'static str,
    pub lock_form: Option<LockForm>,
    /// `None` until WhatsApp sent the list.
    pub blocked: Option<Vec<Blocked>>,
    pub blocked_loading: bool,
    pub blocked_failed: bool,
    /// The chat background image, when one was chosen.
    pub wallpaper_image: Option<String>,
    /// The theme file in use, and every theme file found: (file, name).
    pub custom_theme: Option<String>,
    pub themes: Vec<(String, String)>,
    /// The interface language in use, the stored choice (none follows the
    /// system), and every language offered: (code, own name).
    pub locale: String,
    pub language: Option<String>,
    pub languages: Vec<(String, &'static str)>,
    /// A custom theme's colours as CSS variables, while one is in use.
    pub palette: Option<Vec<(&'static str, String)>>,
    /// Settings › Znga (REQ-ZCHAT-007).
    pub znga: crate::znga::ZngaSettings,
}

/// A locale's code, as settings store it (`en`, `pt-BR`, …).
fn code(locale: zapfast::i18n::Locale) -> String {
    serde_json::to_value(locale)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "en".to_owned())
}

fn css(color: egui::Color32) -> String {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgba({r}, {g}, {b}, {:.3})", f32::from(a) / 255.0)
    }
}

/// A custom theme's palette on the page's colour tokens (tokens.css).
fn palette(p: &zapfast::theme::Palette) -> Vec<(&'static str, String)> {
    vec![
        ("--sidebar-bg", css(p.window)),
        ("--panel-bg", css(p.panel)),
        ("--conversation-header-bg", css(p.panel)),
        ("--empty-bg", css(p.panel)),
        ("--conversation-bg", css(p.chat)),
        ("--search-bg", css(p.surface)),
        ("--input-bg", css(p.surface)),
        ("--menu-bg", css(p.surface)),
        ("--round-button-bg", css(p.surface)),
        ("--header-pill-bg", css(p.surface)),
        ("--composer-button-bg", css(p.surface)),
        ("--menu-hover", css(p.surface_hover)),
        ("--row-hover", css(p.surface_hover)),
        ("--row-selected", css(p.surface_active)),
        ("--sidebar-selected", css(p.surface_active)),
        ("--divider", css(p.outline)),
        ("--panel-border", css(p.outline)),
        ("--conversation-header-border", css(p.outline)),
        ("--text-primary", css(p.text)),
        ("--text-secondary", css(p.secondary)),
        ("--text-section", css(p.dim)),
        ("--icon", css(p.secondary)),
        ("--accent", css(p.accent)),
        ("--unread", css(p.accent)),
        ("--new-chat-bg", css(p.accent)),
        ("--accent-text", css(p.accent)),
        ("--unread-time", css(p.accent)),
        ("--accent-fg", css(p.on_accent)),
        ("--unread-fg", css(p.on_accent)),
        ("--danger", css(p.danger)),
        ("--overlay", css(p.overlay)),
        ("--bubble-in", css(p.bubble_in)),
        ("--bubble-out", css(p.bubble_out)),
        ("--tick-read", css(p.read)),
        ("--date-chip-bg", css(p.surface)),
    ]
}

fn snake<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn swatches(colors: &[zapfast::settings::WallpaperColor]) -> Vec<Swatch> {
    colors
        .iter()
        .map(|color| Swatch {
            id: snake(color),
            label: color.label(),
            color: color
                .rgb()
                .map(|[r, g, b]| format!("#{r:02x}{g:02x}{b:02x}")),
        })
        .collect()
}

fn sound(sound: &zapfast::settings::NotificationSound) -> &'static str {
    use zapfast::settings::NotificationSound;
    match sound {
        NotificationSound::Receive => "receive",
        NotificationSound::Alert => "alert",
        NotificationSound::System => "system",
        NotificationSound::None => "none",
        NotificationSound::Custom(_) => "custom",
    }
}

fn settings(app: &mut App) -> SettingsView {
    use zapfast::app_lock::{FormError, FormMode};
    use zapfast::settings::{AutoLock, ThemeChoice, WallpaperColor};
    let lock_form = app.app_lock.form.as_ref().map(|form| LockForm {
        mode: match form.mode {
            FormMode::Set => "set",
            FormMode::Change => "change",
            FormMode::TurnOff => "turnOff",
        },
        error: form.error.map(|error| match error {
            FormError::TooShort => "tooShort",
            FormError::Mismatch => "mismatch",
            FormError::WrongCurrent => "wrongCurrent",
        }),
        busy: form.busy,
    });
    let wallpaper_image = media(app.account().settings.wallpaper_image.clone());
    let custom_theme = app.settings.custom_theme.clone();
    let themes = app
        .custom_themes
        .picker_themes()
        .map(|theme| {
            (
                theme.filename.clone(),
                fastframe_theme::display_name(&theme.filename).to_owned(),
            )
        })
        .collect();
    let list = &app.account().blocklist;
    let (ids, blocked_loading, blocked_failed) = (list.ids.clone(), list.loading, list.failed);
    let blocked = ids.map(|ids| {
        let mut ids: Vec<String> = ids.into_iter().collect();
        ids.sort();
        let mut rows: Vec<Blocked> = ids
            .into_iter()
            .map(|id| Blocked {
                title: app.display_name(&id),
                avatar: media(app.avatar(&id)),
                id,
            })
            .collect();
        rows.sort_by_key(|row| row.title.to_lowercase());
        rows
    });
    let s = &app.settings;
    let account = &app.account().settings;
    SettingsView {
        // A custom theme carries its own light or dark.
        theme: match (&s.custom_theme, s.theme) {
            (Some(_), _) if app.palette.dark => "dark",
            (Some(_), _) => "light",
            (None, ThemeChoice::Dark) => "dark",
            (None, ThemeChoice::Light) => "light",
            (None, ThemeChoice::System | ThemeChoice::Messages) => "system",
        },
        zoom: s.zoom,
        show_wallpaper: s.show_wallpaper,
        enter_sends: s.enter_sends,
        show_shortcut_hints: s.show_shortcut_hints,
        font: match s.font {
            zapfast::settings::FontChoice::System => "system",
            zapfast::settings::FontChoice::Inter => "inter",
        },
        auto_download: account.auto_download,
        history_prefetch: account.history_prefetch,
        keep_chats_archived: s.keep_chats_archived,
        pause_other_media: s.pause_other_media,
        notifications: account.notifications,
        notification_content: s.notification_content,
        group_sounds: s.group_sounds,
        message_sound: sound(&s.message_sound),
        mention_sound: sound(&s.mention_sound),
        send_read_receipts: account.send_read_receipts,
        send_typing: account.send_typing,
        keep_running: s.keep_running_in_background,
        start_with_system: app.start_with_system,
        check_for_updates: s.check_for_updates,
        download_updates: s.download_updates_automatically,
        proxy: s.proxy.clone(),
        name: app.me_name.clone(),
        about: app.me_about.clone(),
        phone: app
            .me
            .as_deref()
            .and_then(zapfast::model::phone_of)
            .map(|digits| format!("+{digits}")),
        version: env!("CARGO_PKG_VERSION"),
        files: files_view(app),
        wallpaper_light: snake(&app.settings.wallpaper_color),
        wallpaper_dark: snake(&app.settings.dark_wallpaper_color),
        swatches_light: swatches(&WallpaperColor::LIGHT),
        swatches_dark: swatches(&WallpaperColor::DARK),
        chat_lock_code: app.settings.chat_lock_code_hash.is_some(),
        app_lock: app.settings.app_lock_hash.is_some(),
        auto_lock: match app.settings.app_lock_after {
            AutoLock::OneMinute => "oneMinute",
            AutoLock::FifteenMinutes => "fifteenMinutes",
            AutoLock::OneHour => "oneHour",
        },
        lock_form,
        blocked,
        blocked_loading,
        blocked_failed,
        wallpaper_image,
        custom_theme: custom_theme.clone(),
        themes,
        palette: custom_theme.is_some().then(|| palette(&app.palette)),
        locale: code(app.locale),
        language: app.settings.interface_language.map(code),
        languages: zapfast::i18n::Locale::ALL
            .iter()
            .map(|l| (code(*l), l.label()))
            .collect(),
        znga: crate::znga::settings(app),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StarredRow {
    pub chat: String,
    pub chat_title: String,
    pub msg: Msg,
    /// When it was starred, as egui writes it beside the chat's name.
    pub moment: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRow {
    pub id: String,
    pub chat: String,
    /// The chat's name, with "(2)" when calls fold into one row.
    pub title: String,
    /// "Missed voice call", "Video call · 12 min 34 s", as egui writes it.
    pub summary: String,
    /// The time as the egui history writes it.
    pub stamp: String,
    /// The call never connected: its glyph is red (CallStatus::is_bad).
    pub bad: bool,
    pub avatar: Option<String>,
    pub group: bool,
    pub time: i64,
    pub outgoing: bool,
    pub missed: bool,
    pub video: bool,
    /// Seconds the call was up; zero when it never was.
    pub duration: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub channel: bool,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub member_count: usize,
    pub members: Vec<Member>,
    /// Groups this contact shares with us: (id, title).
    pub common_groups: Vec<(String, String)>,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub favorite: bool,
    pub can_leave: bool,
    pub left: bool,
    /// The disappearing-messages timer in seconds; zero when off.
    pub timer: u32,
    /// The number as egui writes it: +39 333 123 4567.
    pub phone_display: Option<String>,
    /// A contact's presence line (online, last seen), as egui shows it.
    pub status: Option<String>,
    /// The panel offers Block or Unblock (blocking::eligible).
    pub blockable: bool,
    pub blocked: bool,
    /// Blocking can be changed now: linked, with the list loaded.
    pub block_editable: bool,
    /// A contact with a number, not ourselves: Add to contacts or Rename.
    pub contact_editable: bool,
    /// The contact has a saved name: the button reads Rename.
    pub known: bool,
    /// The editor's first and last name, split as egui splits them.
    pub editor_names: (String, String),
    /// "Media, links and docs": the count ("7", "200+") once the listing
    /// is in, and the three newest pictures and videos.
    pub media_count: Option<String>,
    pub media: Vec<MediaCell>,
}

/// One picture of the info panel's media strip.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCell {
    pub id: String,
    /// The file when downloaded, else the preview sent with it.
    pub src: Option<String>,
    pub video: bool,
    pub seconds: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Link {
    Starting,
    Unlinked {
        qr: Option<String>,
        code: Option<String>,
    },
    Connecting,
    Connected,
    Disconnected {
        reason: String,
    },
    LoggedOut,
    Failed {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRow {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub time: i64,
    pub unread: u32,
    pub marked_unread: bool,
    pub muted: bool,
    pub pinned: bool,
    pub favorite: bool,
    /// Where the phone puts this favourite; lower first.
    pub favorite_position: u32,
    pub last: Option<Last>,
    /// The time as the egui list writes it: the clock today, "Yesterday",
    /// the weekday this week, else the date.
    pub stamp: String,
    /// "Mira is typing…" or "recording audio…", while someone is.
    pub typing: Option<String>,
    /// Unsent text waiting in this chat.
    pub draft: Option<String>,
    /// An unread message mentions us.
    pub mention: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Last {
    pub from_me: bool,
    pub sender: Option<String>,
    pub text: String,
    pub status: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Open {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub complete: bool,
    pub messages: Vec<Msg>,
    /// The composer offers Znga's tools here (REQ-ZCHAT-004.A1).
    pub znga_tools: bool,
    /// The line under the name: online, typing, last seen, the members.
    pub subtitle: String,
    /// The subtitle is in the accent (online, typing).
    pub subtitle_accent: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Msg {
    pub id: String,
    pub from_me: bool,
    pub sender: Option<String>,
    pub sender_id: String,
    /// The sender's hue: a group sender's name takes its colour.
    pub sender_hue: f32,
    pub time: i64,
    pub kind: String,
    pub text: String,
    pub status: &'static str,
    pub edited: bool,
    pub forwarded: bool,
    pub quoted: Option<Quote>,
    pub reactions: Vec<(String, u32)>,
    /// We reacted: the pill takes an accent outline.
    pub reacted: bool,
    /// The time as egui writes it in a bubble.
    pub clock: String,
    /// Starred: a star beside the time.
    pub starred: bool,
    /// A picture, video, voice note, audio or document, drawn in place
    /// (REQ-WEBUI-004.A4).
    pub media: Option<MediaView>,
    /// A link preview under the text.
    pub preview: Option<PreviewView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaView {
    /// The downloaded file, when there is one.
    pub url: Option<String>,
    /// The small preview sent with it, as a `data:` address.
    pub thumb: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size: u64,
    pub mime: String,
    /// A document's file name.
    pub name: Option<String>,
    pub pages: Option<u32>,
    pub seconds: Option<u32>,
    /// 64 bars, 0 to 100, for a voice note.
    pub waveform: Vec<u8>,
    pub voice: bool,
    pub gif: bool,
    /// A round video message.
    pub round: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewView {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumb: Option<String>,
}

/// JPEG bytes as a `data:` address.
fn jpeg(bytes: &[u8]) -> String {
    use base64::Engine as _;
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// What a bubble draws besides text, and the text it shows (a caption, or
/// the message's own words).
fn media_of(message: &Message) -> (Option<MediaView>, Option<PreviewView>, Option<String>) {
    let thumb = message.thumbnail.as_deref().map(jpeg);
    let view = |media: &zapfast::model::Media| MediaView {
        url: crate::view::media(media.path.clone()),
        thumb: thumb.clone(),
        width: media.width,
        height: media.height,
        size: media.size,
        mime: media.mime.clone(),
        name: None,
        pages: None,
        seconds: None,
        waveform: Vec::new(),
        voice: false,
        gif: false,
        round: false,
    };
    match &message.content {
        Content::Text { text, preview } => (
            None,
            preview.as_ref().map(|p| PreviewView {
                url: p.url.clone(),
                title: p.title.clone(),
                description: p.description.clone(),
                thumb: thumb.clone(),
            }),
            Some(text.clone()),
        ),
        Content::Image { caption, media, .. } => {
            (Some(view(media)), None, Some(caption.clone().unwrap_or_default()))
        }
        Content::Video {
            caption,
            media,
            seconds,
            gif,
            note,
        } => (
            Some(MediaView {
                seconds: *seconds,
                gif: *gif,
                round: *note,
                ..view(media)
            }),
            None,
            Some(caption.clone().unwrap_or_default()),
        ),
        Content::Audio {
            media,
            seconds,
            voice_note,
            waveform,
        } => (
            Some(MediaView {
                seconds: *seconds,
                voice: *voice_note,
                waveform: waveform.clone(),
                ..view(media)
            }),
            None,
            Some(String::new()),
        ),
        Content::Document {
            media,
            file_name,
            caption,
            pages,
        } => (
            Some(MediaView {
                name: Some(file_name.clone()),
                pages: *pages,
                ..view(media)
            }),
            None,
            Some(caption.clone().unwrap_or_default()),
        ),
        Content::Sticker { media, .. } => (Some(view(media)), None, Some(String::new())),
        _ => (None, None, None),
    }
}

/// Settings › Files (src/ui/settings.rs): where the archive, downloads and
/// log are, and what the media cache holds once measured.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesView {
    pub archive: String,
    pub downloads: String,
    /// The downloads go to a folder the user chose: Use default is offered.
    pub custom_downloads: bool,
    pub log: String,
    /// "Photos, videos, stickers and files <account> downloaded…"
    pub storage_hint: String,
    /// Total and (label, "count · size", bar width as a share of the
    /// row) rows, once measured.
    pub storage_total: Option<String>,
    pub storage_rows: Vec<(String, String, f32)>,
    pub clearing: bool,
    pub can_clear: bool,
}

fn files_view(app: &App) -> FilesView {
    use zapfast::i18n::gettext;
    let locale = app.locale;
    let account = app.account();
    let stats = account.storage_stats;
    let total = stats.map(|stats| stats.bytes_total());
    FilesView {
        archive: account.dirs.archive_db().display().to_string(),
        downloads: app
            .settings
            .download_folder
            .clone()
            .unwrap_or_else(|| account.dirs.media_cache_dir())
            .display()
            .to_string(),
        custom_downloads: app.settings.download_folder.is_some(),
        log: app.dirs.log_file().display().to_string(),
        storage_hint: gettext(
            locale,
            "Photos, videos, stickers and files {account} downloaded to this computer. Clearing them keeps your messages; attachments download again when you open them.",
        )
        .replace("{account}", &account.display_label(locale)),
        storage_total: total.map(zapfast::util::bytes),
        storage_rows: stats
            .map(|s| {
                let total = s.bytes_total();
                [
                    (gettext(locale, "Images"), s.images, s.image_bytes, true),
                    (gettext(locale, "Videos and GIFs"), s.videos, s.video_bytes, true),
                    (gettext(locale, "Stickers"), s.stickers, s.sticker_bytes, true),
                    (gettext(locale, "Other"), s.other, s.other_bytes, false),
                ]
                .into_iter()
                .filter(|(_, count, _, always)| *always || *count > 0)
                .map(|(label, count, bytes, _)| {
                    (
                        label.into_owned(),
                        format!("{count} · {}", zapfast::util::bytes(bytes)),
                        zapfast::model::StorageStats::bar_width(bytes, total, 1000.0) / 1000.0,
                    )
                })
                .collect()
            })
            .unwrap_or_default(),
        clearing: account.storage_clearing,
        can_clear: total.is_some_and(|total| total > 0) && !account.storage_clearing,
    }
}

/// The call screen's state, worded as egui's call screen words it
/// (src/ui/call.rs).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallView {
    pub chat: String,
    pub name: String,
    pub avatar: Option<String>,
    pub video: bool,
    /// incoming, live (placing to active) or over (ended, failed).
    pub phase: &'static str,
    /// "Calling…", "Ringing…", "Call ended"; empty while active, where the
    /// page counts `seconds` up itself.
    pub status: String,
    /// Seconds the call has been up, while active.
    pub seconds: Option<u64>,
    pub failed: bool,
    pub muted: bool,
    pub camera_on: bool,
    /// The call can take video: not a group's voice call.
    pub camera: bool,
    /// Mute and camera answer only once the call is connected.
    pub connected: bool,
    /// The screen is set aside: a strip along the bottom leads back.
    pub hidden: bool,
    /// The devices the pickers offer, (id, label), and the ones chosen;
    /// `None` is the system default.
    pub microphones: Vec<(String, String)>,
    pub speakers: Vec<(String, String)>,
    pub cameras: Vec<(String, String)>,
    pub microphone: Option<String>,
    pub speaker: Option<String>,
    pub camera_device: Option<String>,
}

fn call_view(app: &mut App) -> Option<CallView> {
    use zapfast::calls::CallPhase;
    let call = app.call.clone()?;
    let name = app.call_name();
    let avatar = media(app.call_avatar());
    let caller = app.call_caller_name();
    let phase = match call.phase {
        CallPhase::Incoming => "incoming",
        CallPhase::Ended | CallPhase::Failed => "over",
        _ => "live",
    };
    let hidden = app.call_surface_hidden;
    let seconds = (call.phase == CallPhase::Active)
        .then(|| call.started.map(|at| at.elapsed().as_secs()))
        .flatten();
    let status = if call.phase == CallPhase::Incoming {
        zapfast::ui::call::incoming_text(app.locale, &call, caller.as_deref())
    } else if seconds.is_some() {
        String::new()
    } else {
        zapfast::ui::call::status(app.locale, &call)
    };
    Some(CallView {
        chat: call.chat.clone(),
        name,
        avatar,
        video: call.video,
        phase,
        status,
        seconds,
        failed: call.phase == CallPhase::Failed,
        muted: call.muted,
        camera_on: call.camera_on,
        camera: call.group.is_none() || call.video,
        connected: call.phase.is_connected(),
        hidden,
        microphones: app.call_devices.microphones.iter().map(|d| (d.id.clone(), d.label.clone())).collect(),
        speakers: app.call_devices.speakers.iter().map(|d| (d.id.clone(), d.label.clone())).collect(),
        cameras: app.call_devices.cameras.iter().map(|d| (d.id.clone(), d.label.clone())).collect(),
        microphone: call.microphone.clone(),
        speaker: call.speaker.clone(),
        camera_device: call.camera.clone(),
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// "You", or the name egui gives the sender (~push name when unsaved).
    pub author: String,
    pub text: String,
    /// The sender's hue (util::hue): bar and name take its colour.
    pub hue: f32,
    /// Quotes one of our own messages: drawn in the accent.
    pub mine: bool,
}

fn status(delivery: Delivery) -> &'static str {
    match delivery {
        Delivery::None => "none",
        Delivery::Pending => "pending",
        Delivery::Sent => "sent",
        Delivery::Delivered => "delivered",
        Delivery::Read | Delivery::Played => "read",
        Delivery::Failed => "failed",
    }
}

fn filter(filter: ChatFilter) -> &'static str {
    match filter {
        ChatFilter::All => "all",
        ChatFilter::Unread => "unread",
        ChatFilter::Private => "private",
        ChatFilter::Favorites => "favorites",
        ChatFilter::Groups => "groups",
        ChatFilter::Channels => "channels",
    }
}

pub fn parse_filter(name: &str) -> Option<ChatFilter> {
    Some(match name {
        "all" => ChatFilter::All,
        "unread" => ChatFilter::Unread,
        "private" => ChatFilter::Private,
        "favorites" => ChatFilter::Favorites,
        "groups" => ChatFilter::Groups,
        "channels" => ChatFilter::Channels,
        _ => return None,
    })
}

/// A file the page may load, as a `zapmedia:` address.
fn media(path: Option<std::path::PathBuf>) -> Option<String> {
    let path = path?;
    let text = path.to_str()?;
    Some(format!("http://zapmedia.localhost/{}", encode(text)))
}

fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn row(app: &mut App, chat: &Chat, now: i64) -> ChatRow {
    ChatRow {
        id: chat.id.clone(),
        title: app.chat_title(chat),
        avatar: media(app.avatar(&chat.id)),
        group: chat.is_group(),
        time: chat.last_activity,
        unread: chat.unread,
        marked_unread: chat.marked_unread,
        muted: chat.muted(now),
        pinned: chat.pinned,
        favorite: chat.favorite,
        favorite_position: chat.favorite_position,
        last: chat.last.as_ref().map(|last| Last {
            from_me: last.from_me,
            sender: (chat.is_group() && !last.from_me)
                .then(|| last.sender_name.clone().unwrap_or_default())
                .filter(|name| !name.is_empty()),
            text: last.summary.clone(),
            status: status(last.status),
        }),
        stamp: if chat.last_activity > 0 {
            zapfast::util::chat_stamp(app.locale, chat.last_activity)
        } else {
            String::new()
        },
        typing: typing_line(app, chat),
        draft: app.draft_preview(&chat.id),
        mention: chat.unread > 0 && chat.unread_mention.is_some(),
    }
}

/// The typing line egui writes in a row (src/ui/chats.rs).
fn typing_line(app: &App, chat: &Chat) -> Option<String> {
    use zapfast::i18n::gettext;
    use zapfast::model::ChatActivity;
    let typing = app.typing_in(&chat.id);
    let (_, name, activity) = typing.first()?;
    let recording = *activity == ChatActivity::RecordingAudio;
    let name = name.trim_start_matches('~');
    Some(if chat.is_group() && recording {
        gettext(app.locale, "{name} is recording audio…").replace("{name}", name)
    } else if chat.is_group() {
        format!("{name} is typing…")
    } else if recording {
        gettext(app.locale, "recording audio…").into_owned()
    } else {
        "typing…".to_owned()
    })
}

fn message(app: &App, group: bool, message: &Message) -> Msg {
    let kind = serde_json::to_value(&message.content)
        .ok()
        .and_then(|value| value.get("kind")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "text".into());
    let (media, preview, own) = media_of(message);
    let text = own.unwrap_or_else(|| message.content.summary_in(app.locale));
    let mut reactions: Vec<(String, u32)> = Vec::new();
    for reaction in &message.reactions {
        match reactions
            .iter_mut()
            .find(|(emoji, _)| *emoji == reaction.emoji)
        {
            Some((_, count)) => *count += 1,
            None => reactions.push((reaction.emoji.clone(), 1)),
        }
    }
    Msg {
        id: message.id.clone(),
        from_me: message.from_me,
        sender: (group && !message.from_me).then(|| {
            message
                .sender_name
                .clone()
                .unwrap_or_else(|| message.sender.clone())
        }),
        sender_id: message.sender.clone(),
        time: message.timestamp,
        kind,
        text,
        status: status(message.status),
        edited: message.edited,
        forwarded: message.forwarded,
        quoted: message.quoted.as_ref().map(|quoted| {
            let mine = app.me.as_deref() == Some(quoted.sender.as_str());
            Quote {
                author: if mine {
                    "You".to_owned()
                } else {
                    app.display_name_or(&quoted.sender, quoted.sender_name.as_deref())
                },
                text: quoted.summary_in(app.locale).into_owned(),
                hue: zapfast::util::hue(&quoted.sender),
                mine,
            }
        }),
        sender_hue: zapfast::util::hue(&message.sender),
        reactions,
        reacted: message.reactions.iter().any(|reaction| reaction.from_me),
        clock: zapfast::util::clock(message.timestamp),
        starred: app
            .stars
            .get(&message.chat)
            .is_some_and(|ids| ids.contains(&message.id)),
        media,
        preview,
    }
}

pub fn build(app: &mut App) -> State {
    let now = zapfast::util::now();
    let link = match &app.link {
        LinkStatus::Starting => Link::Starting,
        LinkStatus::Unlinked { qr, pair_code, .. } => Link::Unlinked {
            qr: qr.clone(),
            code: pair_code.clone(),
        },
        LinkStatus::Connecting => Link::Connecting,
        LinkStatus::Connected => Link::Connected,
        LinkStatus::Disconnected { reason } => Link::Disconnected {
            reason: reason.clone(),
        },
        LinkStatus::LoggedOut => Link::LoggedOut,
        LinkStatus::Failed(reason) => Link::Failed {
            reason: reason.clone(),
        },
    };
    if app.app_lock.is_locked() {
        use zapfast::app_lock::Forgetting;
        let lock = LockView {
            wrong: app.app_lock.wrong,
            wait: app.app_lock.wait_left().map(|left| left.as_secs() + 1),
            checking: app.app_lock.checking(),
            forgetting: match app.app_lock.forgetting {
                Forgetting::No => "no",
                Forgetting::Confirming => "confirming",
                Forgetting::Unlinking => "unlinking",
            },
        };
        let mut settings = settings(app);
        // Nothing about the account crosses to the page while locked.
        settings.name = None;
        settings.about = None;
        settings.phone = None;
        settings.blocked = None;
        settings.znga.twin.clear();
        settings.znga.excluded.clear();
        settings.znga.facts.clear();
        settings.znga.open_promises.clear();
        settings.znga.audit.clear();
        return State {
            link,
            call: None,
            me: None,
            me_avatar: None,
            filters: Vec::new(),
            filter: filter(app.chat_filter),
            search: String::new(),
            chats: Vec::new(),
            znga_row: None,
            znga: None,
            offer: None,
            unread_chats: 0,
            open: None,
            section: "chats",
            starred: Vec::new(),
            calls: Vec::new(),
            call_filter: "all",
            calls_search: String::new(),
            info: None,
            settings,
            lock: Some(lock),
            locked: LockedView::default(),
        };
    }
    let visible: Vec<Chat> = app.visible_chats().into_iter().cloned().collect();
    let chats = visible.iter().map(|chat| row(app, chat, now)).collect();
    let open = app.open_chat.clone().and_then(|id| {
        let chat = app.chat(&id)?.clone();
        let group = chat.is_group();
        let title = app.chat_title(&chat);
        let avatar = media(app.avatar(&id));
        let conversation = app.conversations.get(&id);
        let (messages, complete) = conversation
            .map(|conversation| {
                let start = conversation.messages.len().saturating_sub(WINDOW);
                (
                    conversation.messages[start..]
                        .iter()
                        .map(|m| message(app, group, m))
                        .collect(),
                    conversation.complete && start == 0,
                )
            })
            .unwrap_or_default();
        let znga_tools = zapfast::znga::chat::tools_for(app, &id);
        let (subtitle, colour) = zapfast::ui::conversation::subtitle(app, &chat);
        let subtitle_accent = colour == app.palette.accent;
        Some(Open {
            subtitle,
            subtitle_accent,
            id,
            title,
            avatar,
            group,
            complete,
            messages,
            znga_tools,
        })
    });
    let locked = LockedView {
        offered: app.locked_count() > 0 || app.locked_folder_open(),
        open: app.locked_folder_open(),
        prompt: matches!(app.dialog, Some(zapfast::model::Dialog::UnlockLockedChats)).then(|| {
            LockedPrompt {
                setup: app.settings.chat_lock_code_hash.is_none(),
                error: app.chat_lock_error,
            }
        }),
    };
    let section = if app.page == Page::Calls {
        "calls"
    } else if app.locked_folder_open() && !app.show_starred {
        "locked"
    } else if app.show_starred {
        "starred"
    } else if app.show_archived {
        "archived"
    } else if app.chat_filter == ChatFilter::Favorites && app.label_filter.is_none() {
        "favourites"
    } else {
        "chats"
    };
    let starred = if app.show_starred {
        let rows: Vec<zapfast::archive::Starred> = app.starred.clone();
        rows.iter()
            // Stars in locked chats stay behind the lock, as in egui.
            .filter(|row| {
                !app.chat(&row.message.chat)
                    .is_some_and(|chat| chat.locked)
            })
            .map(|row| {
                let chat = app.chat(&row.message.chat).cloned();
                let group = chat.as_ref().is_some_and(Chat::is_group);
                StarredRow {
                    chat: row.message.chat.clone(),
                    chat_title: chat
                        .as_ref()
                        .map(|chat| app.chat_title(chat))
                        .unwrap_or_else(|| app.chat_title_of(&row.message.chat)),
                    msg: message(app, group, &row.message),
                    moment: zapfast::util::moment_stamp(app.locale, row.starred_at),
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let calls = if app.page == Page::Calls {
        // egui's history: locked chats' calls left out while their folder
        // is closed, alike calls folded, then the Missed chip and the search.
        use zapfast::ui::calls;
        let visible = calls::visible_calls(app);
        let missed_only = app.call_filter == zapfast::model::CallFilter::Missed;
        let needle = zapfast::util::search_key(app.calls_search.trim());
        let rows: Vec<Vec<Message>> = calls::rows(&visible)
            .into_iter()
            .filter(|row| {
                (!missed_only || calls::Entry::of(row[0]).is_some_and(|e| e.missed()))
                    && calls::matches(app, &row[0].chat, &needle)
            })
            .map(|row| row.into_iter().cloned().collect())
            .collect();
        rows.iter().map(|row| call_row(app, row)).collect()
    } else {
        Vec::new()
    };
    let info = app
        .info
        .as_ref()
        .map(|panel| panel.chat.clone())
        .map(|id| info(app, &id, now));
    let me_avatar = app.me.clone().and_then(|me| media(app.avatar(&me)));
    let filters = ChatFilter::EVERY
        .iter()
        .map(|f| FilterChip {
            id: filter(*f),
            label: f.label(app.locale).into_owned(),
            count: match f {
                ChatFilter::All => 0,
                _ => app.unread_chats(*f),
            },
        })
        .collect();
    let call = call_view(app);
    State {
        link,
        call,
        me: app.me.clone(),
        me_avatar,
        filters,
        filter: filter(app.chat_filter),
        search: app.search.clone(),
        chats,
        znga_row: crate::znga::row(app),
        znga: crate::znga::chat_view(app),
        offer: crate::znga::offer(app),
        open,
        section,
        unread_chats: app.unread_chat_count(),
        starred,
        calls,
        call_filter: match app.call_filter {
            zapfast::model::CallFilter::All => "all",
            zapfast::model::CallFilter::Missed => "missed",
        },
        calls_search: app.calls_search.clone(),
        info,
        settings: settings(app),
        lock: None,
        locked,
    }
}

/// One history row as egui's Calls page builds it (src/ui/calls.rs):
/// consecutive alike calls folded under the newest.
fn call_row(app: &mut App, row: &[Message]) -> CallRow {
    use zapfast::ui::calls;
    let entry = &row[0];
    let call = calls::Entry::of(entry);
    let (video, duration, group_call) = match &entry.content {
        Content::Call {
            video,
            duration,
            group,
            ..
        } => (*video, *duration, *group),
        _ => (false, 0, false),
    };
    let chat = app.chat(&entry.chat).cloned();
    let mut title = calls::row_name(app, &entry.chat);
    if row.len() > 1 {
        title = format!("{title} ({})", row.len());
    }
    CallRow {
        id: entry.id.clone(),
        chat: entry.chat.clone(),
        title,
        summary: call
            .as_ref()
            .map(|call| calls::summary(call, app.locale))
            .unwrap_or_default(),
        stamp: zapfast::util::chat_stamp(app.locale, entry.timestamp),
        bad: call.as_ref().is_some_and(|call| call.status.is_bad()),
        avatar: media(app.avatar(&entry.chat)),
        group: group_call || chat.as_ref().is_some_and(Chat::is_group),
        time: entry.timestamp,
        outgoing: entry.from_me,
        missed: call.as_ref().is_some_and(calls::Entry::missed),
        video,
        duration,
    }
}

fn info(app: &mut App, id: &str, now: i64) -> Info {
    // Group members may have no chat of their own.
    let chat = app
        .chat(id)
        .cloned()
        .unwrap_or_else(|| Chat::new(id.to_owned(), app.display_name(id)));
    let ours: Vec<String> = app.our_ids().into_iter().map(str::to_owned).collect();
    let ours_ref: Vec<&str> = ours.iter().map(String::as_str).collect();
    let roles = zapfast::group_admin::Roles::of(&chat);
    let members = if chat.is_group() {
        app.participant_list(&chat)
            .into_iter()
            .map(|(member, name)| Member {
                avatar: media(app.avatar(&member)),
                admin: roles.is_admin(&member),
                id: member,
                name,
            })
            .collect()
    } else {
        Vec::new()
    };
    let common_groups = if chat.is_group() || chat.is_channel() || ours.iter().any(|o| o == id) {
        Vec::new()
    } else {
        let groups: Vec<Chat> = app
            .chats
            .iter()
            .filter(|group| group.is_group() && group.participants.iter().any(|p| p == id))
            .cloned()
            .collect();
        groups
            .iter()
            .map(|group| (group.id.clone(), app.chat_title(group)))
            .collect()
    };
    Info {
        id: chat.id.clone(),
        title: app.chat_title(&chat),
        avatar: media(app.avatar(id)),
        group: chat.is_group(),
        channel: chat.is_channel(),
        phone: chat.phone().map(str::to_owned),
        description: chat
            .group_description
            .clone()
            .filter(|text| !text.trim().is_empty()),
        member_count: chat.participants.len(),
        members,
        common_groups,
        muted: chat.muted(now),
        pinned: chat.pinned,
        archived: chat.archived,
        favorite: chat.favorite,
        can_leave: chat.can_leave(&ours_ref),
        left: chat.left,
        timer: chat.ephemeral_expiration.unwrap_or(0),
        phone_display: chat
            .phone()
            .map(zapfast::util::phone)
            .filter(|phone| !phone.is_empty()),
        status: (!chat.is_group() && !chat.is_channel())
            .then(|| zapfast::ui::conversation::subtitle(app, &chat).0)
            .filter(|line| !line.is_empty() && Some(line.as_str()) != chat.phone().map(zapfast::util::phone).as_deref()),
        blockable: zapfast::blocking::eligible(id, app.me.as_deref(), app.me_lid.as_deref()),
        blocked: app.blocklist.contains(id),
        block_editable: app.is_connected() && app.blocklist.editable(),
        contact_editable: !chat.is_group()
            && !chat.is_channel()
            && chat.phone().is_some()
            && !ours.iter().any(|mine| mine == id),
        known: app
            .contacts
            .get(id)
            .and_then(|contact| contact.full_name.as_deref())
            .is_some_and(|full| !full.is_empty()),
        media_count: app.info_media.as_ref().filter(|m| m.chat == id).map(|m| {
            let count = m.media.len() + m.docs.len() + m.links.len();
            if m.truncated() {
                format!("{count}+")
            } else {
                count.to_string()
            }
        }),
        media: app
            .info_media
            .as_ref()
            .filter(|m| m.chat == id)
            .map(|m| {
                m.media
                    .iter()
                    .take(3)
                    .map(|message| {
                        let (view, _, _) = media_of(message);
                        let video = view.as_ref().is_some_and(|v| v.mime.starts_with("video/"));
                        MediaCell {
                            id: message.id.clone(),
                            src: view.as_ref().and_then(|v| {
                                if video { v.thumb.clone() } else { v.url.clone().or_else(|| v.thumb.clone()) }
                            }),
                            video,
                            seconds: view.and_then(|v| v.seconds),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default(),
        editor_names: zapfast::util::editor_names(
            app.chat_title(&chat).trim_start_matches('~'),
            app.contacts.get(id).and_then(zapfast::model::Contact::first_name),
        ),
    }
}
