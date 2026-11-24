//! Interface texts in German, Turkish and English.
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    De,
    Tr,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::De => "de",
            Lang::Tr => "tr",
            Lang::En => "en",
        }
    }

    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "de" => Some(Lang::De),
            "tr" => Some(Lang::Tr),
            "en" => Some(Lang::En),
            _ => None,
        }
    }

    /// `LANGUAGE=de|tr|en` in the UI configuration; German otherwise.
    pub fn from_config(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| {
                text.lines()
                    .filter_map(|line| line.trim().strip_prefix("LANGUAGE="))
                    .next_back()
                    .and_then(|value| Lang::parse(value.trim()))
            })
            .unwrap_or(Lang::De)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Text {
    AppName,
    SetupTitle,
    SetupIntro,
    SetupCode,
    NewPassword,
    RepeatPassword,
    PasswordHint,
    Save,
    LoginTitle,
    Password,
    LogIn,
    LogOut,
    HomeTitle,
    HomeIntro,
    WrongCode,
    CodeExpired,
    WrongPassword,
    PasswordTooShort,
    PasswordTooLong,
    PasswordMismatch,
    TooManyAttempts,
    RequestRejected,
    SavingFailed,
    PasswordSaved,
}

pub fn t(lang: Lang, text: Text) -> &'static str {
    use Text::*;
    match (lang, text) {
        (_, AppName) => "Dede TV",

        (Lang::De, SetupTitle) => "Einrichtung",
        (Lang::Tr, SetupTitle) => "Kurulum",
        (Lang::En, SetupTitle) => "Setup",

        (Lang::De, SetupIntro) => "Der Fernseher zeigt einen achtstelligen Einrichtungscode. Geben Sie ihn hier ein und legen Sie ein Passwort für die Verwaltung fest.",
        (Lang::Tr, SetupIntro) => "Televizyon sekiz haneli bir kurulum kodu gösteriyor. Kodu buraya girin ve yönetim için bir şifre belirleyin.",
        (Lang::En, SetupIntro) => "The TV shows an eight-digit setup code. Enter it here and choose a password for the administration.",

        (Lang::De, SetupCode) => "Einrichtungscode vom Fernseher",
        (Lang::Tr, SetupCode) => "Televizyondaki kurulum kodu",
        (Lang::En, SetupCode) => "Setup code from the TV",

        (Lang::De, NewPassword) => "Neues Passwort",
        (Lang::Tr, NewPassword) => "Yeni şifre",
        (Lang::En, NewPassword) => "New password",

        (Lang::De, RepeatPassword) => "Passwort wiederholen",
        (Lang::Tr, RepeatPassword) => "Şifreyi tekrarlayın",
        (Lang::En, RepeatPassword) => "Repeat password",

        (Lang::De, PasswordHint) => "Mindestens 10 Zeichen.",
        (Lang::Tr, PasswordHint) => "En az 10 karakter.",
        (Lang::En, PasswordHint) => "At least 10 characters.",

        (Lang::De, Save) => "Speichern",
        (Lang::Tr, Save) => "Kaydet",
        (Lang::En, Save) => "Save",

        (Lang::De, LoginTitle) => "Anmelden",
        (Lang::Tr, LoginTitle) => "Giriş",
        (Lang::En, LoginTitle) => "Sign in",

        (Lang::De, Password) => "Passwort",
        (Lang::Tr, Password) => "Şifre",
        (Lang::En, Password) => "Password",

        (Lang::De, LogIn) => "Anmelden",
        (Lang::Tr, LogIn) => "Giriş yap",
        (Lang::En, LogIn) => "Sign in",

        (Lang::De, LogOut) => "Abmelden",
        (Lang::Tr, LogOut) => "Çıkış",
        (Lang::En, LogOut) => "Sign out",

        (Lang::De, HomeTitle) => "Verwaltung",
        (Lang::Tr, HomeTitle) => "Yönetim",
        (Lang::En, HomeTitle) => "Administration",

        (Lang::De, HomeIntro) => "Sie sind angemeldet. Hier verwalten Sie Sender, eigene Filme, Sprache und Fernzugriff.",
        (Lang::Tr, HomeIntro) => "Giriş yaptınız. Buradan kanalları, kendi filmlerinizi, dili ve uzaktan erişimi yönetebilirsiniz.",
        (Lang::En, HomeIntro) => "You are signed in. Manage channels, your own films, language and remote access here.",

        (Lang::De, WrongCode) => "Der Code stimmt nicht. Bitte prüfen Sie die Ziffern auf dem Fernseher.",
        (Lang::Tr, WrongCode) => "Kod yanlış. Lütfen televizyondaki rakamları kontrol edin.",
        (Lang::En, WrongCode) => "The code is wrong. Please check the digits on the TV.",

        (Lang::De, CodeExpired) => "Der Code ist abgelaufen. Der Fernseher zeigt jetzt einen neuen Code.",
        (Lang::Tr, CodeExpired) => "Kodun süresi doldu. Televizyon şimdi yeni bir kod gösteriyor.",
        (Lang::En, CodeExpired) => "The code has expired. The TV now shows a new code.",

        (Lang::De, WrongPassword) => "Das Passwort stimmt nicht.",
        (Lang::Tr, WrongPassword) => "Şifre yanlış.",
        (Lang::En, WrongPassword) => "The password is wrong.",

        (Lang::De, PasswordTooShort) => "Das Passwort ist zu kurz (mindestens 10 Zeichen).",
        (Lang::Tr, PasswordTooShort) => "Şifre çok kısa (en az 10 karakter).",
        (Lang::En, PasswordTooShort) => "The password is too short (at least 10 characters).",

        (Lang::De, PasswordTooLong) => "Das Passwort ist zu lang.",
        (Lang::Tr, PasswordTooLong) => "Şifre çok uzun.",
        (Lang::En, PasswordTooLong) => "The password is too long.",

        (Lang::De, PasswordMismatch) => "Die beiden Passwörter sind nicht gleich.",
        (Lang::Tr, PasswordMismatch) => "İki şifre aynı değil.",
        (Lang::En, PasswordMismatch) => "The two passwords do not match.",

        (Lang::De, TooManyAttempts) => "Zu viele Versuche. Bitte warten Sie 15 Minuten.",
        (Lang::Tr, TooManyAttempts) => "Çok fazla deneme. Lütfen 15 dakika bekleyin.",
        (Lang::En, TooManyAttempts) => "Too many attempts. Please wait 15 minutes.",

        (Lang::De, RequestRejected) => "Die Anfrage wurde abgelehnt. Bitte laden Sie die Seite neu.",
        (Lang::Tr, RequestRejected) => "İstek reddedildi. Lütfen sayfayı yeniden yükleyin.",
        (Lang::En, RequestRejected) => "The request was rejected. Please reload the page.",

        (Lang::De, SavingFailed) => "Speichern fehlgeschlagen. Bitte versuchen Sie es erneut.",
        (Lang::Tr, SavingFailed) => "Kaydetme başarısız. Lütfen tekrar deneyin.",
        (Lang::En, SavingFailed) => "Saving failed. Please try again.",

        (Lang::De, PasswordSaved) => "Das Passwort ist gespeichert.",
        (Lang::Tr, PasswordSaved) => "Şifre kaydedildi.",
        (Lang::En, PasswordSaved) => "The password has been saved.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_from_config() {
        let dir = std::env::temp_dir().join(format!("dede-i18n-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("ui.conf");
        assert_eq!(Lang::from_config(&file), Lang::De);
        std::fs::write(&file, "# Sprache\nLANGUAGE=tr\n").unwrap();
        assert_eq!(Lang::from_config(&file), Lang::Tr);
        std::fs::write(&file, "LANGUAGE=xx\n").unwrap();
        assert_eq!(Lang::from_config(&file), Lang::De);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn every_text_differs_per_language_where_translated() {
        assert_ne!(t(Lang::De, Text::LogIn), t(Lang::Tr, Text::LogIn));
        assert_ne!(t(Lang::Tr, Text::LogIn), t(Lang::En, Text::LogIn));
    }
}
