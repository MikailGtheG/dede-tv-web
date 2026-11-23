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

pub fn home(lang: Lang, csrf: &str, message: Option<Notice>) -> String {
    let logout = format!(
        r#"<form method="post" action="/logout">{}<button class="link" type="submit">{}</button></form>"#,
        hidden("csrf", csrf),
        escape(t(lang, Text::LogOut))
    );
    let body = format!(
        "<h1>{}</h1>\n{}<p>{}</p>",
        escape(t(lang, Text::HomeTitle)),
        notice(lang, message),
        escape(t(lang, Text::HomeIntro))
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
