//! Server-rendered pages. Every dynamic value goes through `escape`.
use crate::i18n::{t, Lang, Text};

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub enum Notice {
    Error(Text),
    Ok(Text),
}

fn notice(lang: Lang, notice: Option<Notice>) -> String {
    match notice {
        Some(Notice::Error(text)) => {
            format!(
                r#"<p class="msg" role="alert">{}</p>"#,
                escape(t(lang, text))
            )
        }
        Some(Notice::Ok(text)) => {
            format!(
                r#"<p class="msg ok" role="status">{}</p>"#,
                escape(t(lang, text))
            )
        }
        None => String::new(),
    }
}

fn page(lang: Lang, title: Text, header_extra: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} – Dede TV</title>
<link rel="stylesheet" href="/style.css">
</head>
<body>
<header><strong><b>Dede</b> TV</strong>{header_extra}</header>
<main>
{body}
</main>
</body>
</html>
"#,
        lang = lang.code(),
        title = escape(t(lang, title)),
    )
}

fn hidden(name: &str, value: &str) -> String {
    format!(
        r#"<input type="hidden" name="{}" value="{}">"#,
        escape(name),
        escape(value)
    )
}

pub fn setup(lang: Lang, form_token: &str, message: Option<Notice>) -> String {
    let body = format!(
        r#"<h1>{title}</h1>
<p>{intro}</p>
{notice}
<form method="post" action="/setup">
{token}
<label for="code">{code}</label>
<input id="code" name="code" inputmode="numeric" autocomplete="one-time-code" maxlength="12" required autofocus>
<label for="password">{new}</label>
<input id="password" name="password" type="password" autocomplete="new-password" minlength="10" required>
<p class="hint">{hint}</p>
<label for="repeat">{repeat}</label>
<input id="repeat" name="repeat" type="password" autocomplete="new-password" minlength="10" required>
<button type="submit">{save}</button>
</form>"#,
        title = escape(t(lang, Text::SetupTitle)),
        intro = escape(t(lang, Text::SetupIntro)),
        notice = notice(lang, message),
        token = hidden("form", form_token),
        code = escape(t(lang, Text::SetupCode)),
        new = escape(t(lang, Text::NewPassword)),
        hint = escape(t(lang, Text::PasswordHint)),
        repeat = escape(t(lang, Text::RepeatPassword)),
        save = escape(t(lang, Text::Save)),
    );
    page(lang, Text::SetupTitle, "", &body)
}

pub fn login(lang: Lang, form_token: &str, message: Option<Notice>) -> String {
    let body = format!(
        r#"<h1>{title}</h1>
{notice}
<form method="post" action="/login">
{token}
<label for="password">{password}</label>
<input id="password" name="password" type="password" autocomplete="current-password" required autofocus>
<button type="submit">{login}</button>
</form>"#,
        title = escape(t(lang, Text::LoginTitle)),
        notice = notice(lang, message),
        token = hidden("form", form_token),
        password = escape(t(lang, Text::Password)),
        login = escape(t(lang, Text::LogIn)),
    );
    page(lang, Text::LoginTitle, "", &body)
}

fn logout_form(lang: Lang, csrf: &str) -> String {
    format!(
        r#"<form method="post" action="/logout">{}<button class="link" type="submit">{}</button></form>"#,
        hidden("csrf", csrf),
        escape(t(lang, Text::LogOut))
    )
}

fn post_button(
    action: &str,
    csrf: &str,
    extra: &[(&str, &str)],
    label: &str,
    class: &str,
) -> String {
    let fields: String = extra.iter().map(|(k, v)| hidden(k, v)).collect();
    format!(
        r#"<form method="post" action="{}">{}{}<button class="{}" type="submit">{}</button></form>"#,
        escape(action),
        hidden("csrf", csrf),
        fields,
        class,
        escape(label)
    )
}

pub fn channels(
    lang: Lang,
    csrf: &str,
    list: &[crate::channels::Channel],
    message: Option<Notice>,
) -> String {
    let mut rows = String::new();
    for c in list {
        rows.push_str(&format!(
            r#"<a class="row" href="/channels/{n}"><span class="num">{n:02}</span><span>{name}</span></a>"#,
            n = c.number,
            name = escape(&c.name)
        ));
    }
    if rows.is_empty() {
        rows = format!("<p>{}</p>", escape(t(lang, Text::NoChannels)));
    }
    let body = format!(
        r#"<p><a href="/">← {back}</a></p>
<h1>{title}</h1>
{notice}<p>{intro}</p>
<nav class="rows">{rows}</nav>
<a class="button" href="/channels/new">+ {new}</a>"#,
        back = escape(t(lang, Text::Back)),
        title = escape(t(lang, Text::ChannelsTitle)),
        notice = notice(lang, message),
        intro = escape(t(lang, Text::ChannelsIntro)),
        new = escape(t(lang, Text::NewChannel)),
    );
    page(lang, Text::ChannelsTitle, &logout_form(lang, csrf), &body)
}

pub fn new_channel(
    lang: Lang,
    csrf: &str,
    name: &str,
    number: &str,
    message: Option<Notice>,
) -> String {
    let body = format!(
        r#"<p><a href="/channels">← {back}</a></p>
<h1>{title}</h1>
{notice}
<form method="post" action="/channels/new">
{token}
<label for="name">{name_label}</label>
<input id="name" name="name" value="{name}" maxlength="60" required autofocus>
<label for="number">{number_label}</label>
<input id="number" name="number" value="{number}" inputmode="numeric" maxlength="4" required>
<p class="hint">{hint}</p>
<button type="submit">{create}</button>
</form>"#,
        back = escape(t(lang, Text::Back)),
        title = escape(t(lang, Text::NewChannel)),
        notice = notice(lang, message),
        token = hidden("csrf", csrf),
        name_label = escape(t(lang, Text::ChannelName)),
        name = escape(name),
        number_label = escape(t(lang, Text::ChannelNumber)),
        number = escape(number),
        hint = escape(t(lang, Text::NumberHint)),
        create = escape(t(lang, Text::Create)),
    );
    page(lang, Text::NewChannel, &logout_form(lang, csrf), &body)
}

pub fn channel(
    lang: Lang,
    csrf: &str,
    channel: &crate::channels::Channel,
    films: &[crate::media::Film],
    message: Option<Notice>,
) -> String {
    let base = format!("/channels/{}", channel.number);
    let mut rows = String::new();
    for film in films {
        rows.push_str(&format!(
            r#"<div class="row"><span>{name}</span><span class="size">{mb} MB</span>{delete}</div>"#,
            name = escape(&film.name),
            mb = film.bytes.div_ceil(1024 * 1024),
            delete = post_button(&format!("{base}/films/delete"), csrf, &[("film", &film.name)], t(lang, Text::Delete), "link"),
        ));
    }
    if rows.is_empty() {
        rows = format!("<p>{}</p>", escape(t(lang, Text::NoFilms)));
    }
    let body = format!(
        r#"<p><a href="/channels">← {back}</a></p>
<h1><span class="num">{n:02}</span> {name}</h1>
{notice}
<h2>{films_title}</h2>
<div class="rows">{rows}</div>
<h2>{upload_title}</h2>
<form method="post" action="{base}/upload" enctype="multipart/form-data">
{token}
<input type="file" name="film" accept="video/*,.mkv,.ts" multiple required>
<p class="hint">{upload_hint}</p>
<button type="submit">{upload}</button>
</form>
<h2>{send_title}</h2>
<p>{send_hint}</p>
{send}
<h2>{delete_title}</h2>
<p>{delete_hint}</p>
{delete}"#,
        back = escape(t(lang, Text::Back)),
        n = channel.number,
        name = escape(&channel.name),
        notice = notice(lang, message),
        films_title = escape(t(lang, Text::Films)),
        upload_title = escape(t(lang, Text::UploadFilms)),
        base = escape(&base),
        token = hidden("csrf", csrf),
        upload_hint = escape(t(lang, Text::UploadHint)),
        upload = escape(t(lang, Text::Upload)),
        send_title = escape(t(lang, Text::SendToTv)),
        send_hint = escape(t(lang, Text::SendToTvHint)),
        send = post_button(
            "/apply",
            csrf,
            &[("back", &base)],
            t(lang, Text::SendToTv),
            ""
        ),
        delete_title = escape(t(lang, Text::DeleteChannel)),
        delete_hint = escape(t(lang, Text::DeleteChannelHint)),
        delete = post_button(
            &format!("{base}/delete"),
            csrf,
            &[],
            t(lang, Text::DeleteChannel),
            "danger"
        ),
    );
    page(lang, Text::ChannelsTitle, &logout_form(lang, csrf), &body)
}

pub fn language(lang: Lang, csrf: &str) -> String {
    let mut options = String::new();
    for option in [Lang::De, Lang::Tr, Lang::En] {
        options.push_str(&format!(
            r#"<label class="choice"><input type="radio" name="language" value="{code}"{checked}> {name}</label>"#,
            code = option.code(),
            checked = if option == lang { " checked" } else { "" },
            name = option.native_name(),
        ));
    }
    let body = format!(
        r#"<p><a href="/">← {back}</a></p>
<h1>{title}</h1>
<p>{intro}</p>
<form method="post" action="/language">
{token}
{options}
<button type="submit">{save}</button>
</form>"#,
        back = escape(t(lang, Text::Back)),
        title = escape(t(lang, Text::LanguageTitle)),
        intro = escape(t(lang, Text::LanguageIntro)),
        token = hidden("csrf", csrf),
        save = escape(t(lang, Text::Save)),
    );
    page(lang, Text::LanguageTitle, &logout_form(lang, csrf), &body)
}

pub fn iptv(lang: Lang, csrf: &str, host: Option<&str>, message: Option<Notice>) -> String {
    let current = match host {
        Some(host) => format!(
            "<p>{} <strong>{}</strong></p>",
            escape(t(lang, Text::IptvCurrent)),
            escape(host)
        ),
        None => format!("<p>{}</p>", escape(t(lang, Text::IptvNone))),
    };
    let body = format!(
        r#"<p><a href="/">← {back}</a></p>
<h1>{title}</h1>
{current}{notice}<p>{intro}</p>
<form method="post" action="/iptv">
{token}
<label for="m3u">{m3u}</label>
<input id="m3u" name="m3u_url" type="url" inputmode="url" autocomplete="off" maxlength="2048" required>
<label for="epg">{epg}</label>
<input id="epg" name="epg_url" type="url" inputmode="url" autocomplete="off" maxlength="2048">
<button type="submit">{save}</button>
</form>"#,
        back = escape(t(lang, Text::Back)),
        title = escape(t(lang, Text::IptvTitle)),
        notice = notice(lang, message),
        intro = escape(t(lang, Text::IptvIntro)),
        token = hidden("csrf", csrf),
        m3u = escape(t(lang, Text::IptvM3u)),
        epg = escape(t(lang, Text::IptvEpg)),
        save = escape(t(lang, Text::Save)),
    );
    page(lang, Text::IptvTitle, &logout_form(lang, csrf), &body)
}

pub enum ApplyState {
    Pending,
    Done,
    Failed(String),
}

pub fn applying(lang: Lang, csrf: &str, state: ApplyState, back: &str) -> String {
    let (refresh, content) = match state {
        ApplyState::Pending => (
            r#"<meta http-equiv="refresh" content="3">"#,
            format!("<p>{}</p>", escape(t(lang, Text::ApplyingHint))),
        ),
        ApplyState::Done => (
            "",
            format!(
                r#"<p class="msg ok" role="status">{}</p>"#,
                escape(t(lang, Text::Applied))
            ),
        ),
        ApplyState::Failed(code) => (
            "",
            format!(
                r#"<p class="msg" role="alert">{} {}</p>"#,
                escape(t(lang, Text::ApplyFailed)),
                escape(&code)
            ),
        ),
    };
    let body = format!(
        r#"{refresh}<h1>{title}</h1>{content}<p><a class="button" href="{back}">{back_label}</a></p>"#,
        title = escape(t(lang, Text::Applying)),
        back = escape(back),
        back_label = escape(t(lang, Text::Back)),
    );
    page(lang, Text::Applying, &logout_form(lang, csrf), &body)
}

pub fn home(lang: Lang, csrf: &str, message: Option<Notice>) -> String {
    let logout = format!(
        r#"<form method="post" action="/logout">{}<button class="link" type="submit">{}</button></form>"#,
        hidden("csrf", csrf),
        escape(t(lang, Text::LogOut))
    );
    let body = format!(
        "<h1>{}</h1>\n{}<p>{}</p>\n<nav class=\"rows\"><a class=\"row\" href=\"/channels\">{}</a><a class=\"row\" href=\"/iptv\">{}</a><a class=\"row\" href=\"/language\">{}</a></nav>",
        escape(t(lang, Text::HomeTitle)),
        notice(lang, message),
        escape(t(lang, Text::HomeIntro)),
        escape(t(lang, Text::ChannelsTitle)),
        escape(t(lang, Text::IptvTitle)),
        escape(t(lang, Text::LanguageTitle))
    );
    page(lang, Text::HomeTitle, &logout, &body)
}

pub fn rejected(lang: Lang) -> String {
    let body = format!(
        r#"<p class="msg" role="alert">{}</p><p><a href="/">Dede TV</a></p>"#,
        escape(t(lang, Text::RequestRejected))
    );
    page(lang, Text::AppName, "", &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup_and_attributes() {
        assert_eq!(
            escape(r#"<a href="x" onclick='y'>&</a>"#),
            "&lt;a href=&quot;x&quot; onclick=&#39;y&#39;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    fn pages_carry_tokens_and_language() {
        let html = setup(Lang::Tr, "tok\"en", Some(Notice::Error(Text::WrongCode)));
        assert!(html.contains(r#"<html lang="tr">"#));
        assert!(html.contains(r#"value="tok&quot;en""#));
        assert!(html.contains("Kod yanlış"));
        let home_page = home(Lang::En, "abc", None);
        assert!(home_page.contains(r#"name="csrf" value="abc""#));
        assert!(!home_page.contains("<script"));
    }
}
