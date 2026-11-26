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

    /// The language's own name, shown in the language list.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::De => "Deutsch",
            Lang::Tr => "Türkçe",
            Lang::En => "English",
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
    ChannelsTitle,
    ChannelsIntro,
    NoChannels,
    NewChannel,
    ChannelName,
    ChannelNumber,
    NumberHint,
    Create,
    Films,
    NoFilms,
    UploadFilms,
    UploadHint,
    Upload,
    Delete,
    DeleteChannel,
    DeleteChannelHint,
    SendToTv,
    SendToTvHint,
    Applying,
    ApplyingHint,
    Applied,
    ApplyFailed,
    Back,
    NameInvalid,
    NumberInvalid,
    NumberTaken,
    TooManyChannels,
    UploadDone,
    UploadBadName,
    UploadExists,
    UploadFailed,
    ConfigUnreadable,
    FilmDeleted,
    FolderFailed,
    LanguageTitle,
    LanguageIntro,
    IptvTitle,
    IptvIntro,
    IptvCurrent,
    IptvNone,
    IptvM3u,
    IptvEpg,
    IptvInvalid,
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

        (Lang::De, ChannelsTitle) => "Eigene Sender",
        (Lang::Tr, ChannelsTitle) => "Kendi kanallarım",
        (Lang::En, ChannelsTitle) => "Own channels",

        (Lang::De, ChannelsIntro) => "Ein eigener Sender spielt die Filme aus seinem Ordner rund um die Uhr ab, wie ein Fernsehsender.",
        (Lang::Tr, ChannelsIntro) => "Kendi kanalınız, klasöründeki filmleri bir televizyon kanalı gibi günün her saatinde oynatır.",
        (Lang::En, ChannelsIntro) => "An own channel plays the films in its folder around the clock, like a TV channel.",

        (Lang::De, NoChannels) => "Noch keine eigenen Sender.",
        (Lang::Tr, NoChannels) => "Henüz kendi kanalınız yok.",
        (Lang::En, NoChannels) => "No own channels yet.",

        (Lang::De, NewChannel) => "Neuer Sender",
        (Lang::Tr, NewChannel) => "Yeni kanal",
        (Lang::En, NewChannel) => "New channel",

        (Lang::De, ChannelName) => "Name des Senders",
        (Lang::Tr, ChannelName) => "Kanal adı",
        (Lang::En, ChannelName) => "Channel name",

        (Lang::De, ChannelNumber) => "Sendernummer",
        (Lang::Tr, ChannelNumber) => "Kanal numarası",
        (Lang::En, ChannelNumber) => "Channel number",

        (Lang::De, NumberHint) => "Unter dieser Nummer wählen Sie den Sender auf der Fernbedienung.",
        (Lang::Tr, NumberHint) => "Kanalı kumandada bu numarayla seçersiniz.",
        (Lang::En, NumberHint) => "You select the channel with this number on the remote.",

        (Lang::De, Create) => "Sender anlegen",
        (Lang::Tr, Create) => "Kanalı oluştur",
        (Lang::En, Create) => "Create channel",

        (Lang::De, Films) => "Filme",
        (Lang::Tr, Films) => "Filmler",
        (Lang::En, Films) => "Films",

        (Lang::De, NoFilms) => "Noch keine Filme. Laden Sie unten Filme hoch.",
        (Lang::Tr, NoFilms) => "Henüz film yok. Aşağıdan film yükleyin.",
        (Lang::En, NoFilms) => "No films yet. Upload films below.",

        (Lang::De, UploadFilms) => "Filme hochladen",
        (Lang::Tr, UploadFilms) => "Film yükle",
        (Lang::En, UploadFilms) => "Upload films",

        (Lang::De, UploadHint) => "Videodateien wie MP4 oder MKV. Große Dateien brauchen einige Minuten, bitte die Seite so lange offen lassen.",
        (Lang::Tr, UploadHint) => "MP4 veya MKV gibi video dosyaları. Büyük dosyalar birkaç dakika sürer, lütfen sayfayı bu süre açık bırakın.",
        (Lang::En, UploadHint) => "Video files such as MP4 or MKV. Large files take a few minutes; please keep the page open.",

        (Lang::De, Upload) => "Hochladen",
        (Lang::Tr, Upload) => "Yükle",
        (Lang::En, Upload) => "Upload",

        (Lang::De, Delete) => "Löschen",
        (Lang::Tr, Delete) => "Sil",
        (Lang::En, Delete) => "Delete",

        (Lang::De, DeleteChannel) => "Sender löschen",
        (Lang::Tr, DeleteChannel) => "Kanalı sil",
        (Lang::En, DeleteChannel) => "Delete channel",

        (Lang::De, DeleteChannelHint) => "Die Filme bleiben im Ordner erhalten.",
        (Lang::Tr, DeleteChannelHint) => "Filmler klasörde kalır.",
        (Lang::En, DeleteChannelHint) => "The films stay in the folder.",

        (Lang::De, SendToTv) => "Auf den Fernseher bringen",
        (Lang::Tr, SendToTv) => "Televizyona aktar",
        (Lang::En, SendToTv) => "Send to the TV",

        (Lang::De, SendToTvHint) => "Neue oder gelöschte Filme erscheinen erst nach diesem Schritt. Der Fernseher startet dafür kurz neu.",
        (Lang::Tr, SendToTvHint) => "Yeni veya silinen filmler ancak bu adımdan sonra görünür. Televizyon bunun için kısa süre yeniden başlar.",
        (Lang::En, SendToTvHint) => "New or deleted films appear only after this step. The TV restarts briefly for it.",

        (Lang::De, Applying) => "Wird übernommen …",
        (Lang::Tr, Applying) => "Aktarılıyor …",
        (Lang::En, Applying) => "Applying …",

        (Lang::De, ApplyingHint) => "Der Fernseher startet gleich kurz neu. Diese Seite aktualisiert sich selbst.",
        (Lang::Tr, ApplyingHint) => "Televizyon birazdan kısa süre yeniden başlar. Bu sayfa kendini yeniler.",
        (Lang::En, ApplyingHint) => "The TV will restart briefly. This page refreshes itself.",

        (Lang::De, Applied) => "Fertig. Der Fernseher zeigt die Änderung in etwa einer Minute.",
        (Lang::Tr, Applied) => "Tamam. Televizyon değişikliği yaklaşık bir dakika içinde gösterir.",
        (Lang::En, Applied) => "Done. The TV shows the change in about a minute.",

        (Lang::De, ApplyFailed) => "Übernahme fehlgeschlagen. Fehlercode:",
        (Lang::Tr, ApplyFailed) => "Aktarma başarısız. Hata kodu:",
        (Lang::En, ApplyFailed) => "Applying failed. Error code:",

        (Lang::De, Back) => "Zurück",
        (Lang::Tr, Back) => "Geri",
        (Lang::En, Back) => "Back",

        (Lang::De, NameInvalid) => "Bitte einen neuen Namen ohne Zeichen wie , \" [ ] wählen.",
        (Lang::Tr, NameInvalid) => "Lütfen , \" [ ] gibi karakterler içermeyen yeni bir ad seçin.",
        (Lang::En, NameInvalid) => "Please choose a new name without characters such as , \" [ ].",

        (Lang::De, NumberInvalid) => "Bitte eine Nummer von 1 bis 9999 eingeben.",
        (Lang::Tr, NumberInvalid) => "Lütfen 1 ile 9999 arasında bir numara girin.",
        (Lang::En, NumberInvalid) => "Please enter a number from 1 to 9999.",

        (Lang::De, NumberTaken) => "Diese Nummer hat schon ein anderer eigener Sender.",
        (Lang::Tr, NumberTaken) => "Bu numara başka bir kanalınızda kullanılıyor.",
        (Lang::En, NumberTaken) => "Another own channel already uses this number.",

        (Lang::De, TooManyChannels) => "Es sind höchstens 50 eigene Sender möglich.",
        (Lang::Tr, TooManyChannels) => "En fazla 50 kanal olabilir.",
        (Lang::En, TooManyChannels) => "At most 50 own channels are possible.",

        (Lang::De, UploadDone) => "Hochladen abgeschlossen. Mit „Auf den Fernseher bringen“ erscheinen die Filme im Programm.",
        (Lang::Tr, UploadDone) => "Yükleme tamamlandı. Filmler „Televizyona aktar“ ile yayına girer.",
        (Lang::En, UploadDone) => "Upload finished. “Send to the TV” puts the films on air.",

        (Lang::De, UploadBadName) => "Diese Datei ist nicht erlaubt. Erlaubt sind Videodateien wie .mp4 oder .mkv.",
        (Lang::Tr, UploadBadName) => "Bu dosyaya izin verilmiyor. .mp4 veya .mkv gibi video dosyaları kullanın.",
        (Lang::En, UploadBadName) => "This file is not allowed. Use video files such as .mp4 or .mkv.",

        (Lang::De, UploadExists) => "Ein Film mit diesem Namen ist schon da.",
        (Lang::Tr, UploadExists) => "Bu adla bir film zaten var.",
        (Lang::En, UploadExists) => "A film with this name already exists.",

        (Lang::De, UploadFailed) => "Hochladen fehlgeschlagen. Ist genug Speicherplatz frei?",
        (Lang::Tr, UploadFailed) => "Yükleme başarısız. Yeterli boş alan var mı?",
        (Lang::En, UploadFailed) => "Upload failed. Is there enough free space?",

        (Lang::De, ConfigUnreadable) => "Die Senderliste auf dem Gerät ist nicht lesbar.",
        (Lang::Tr, ConfigUnreadable) => "Cihazdaki kanal listesi okunamıyor.",
        (Lang::En, ConfigUnreadable) => "The channel list on the device cannot be read.",

        (Lang::De, FilmDeleted) => "Film gelöscht. Mit „Auf den Fernseher bringen“ verschwindet er aus dem Programm.",
        (Lang::Tr, FilmDeleted) => "Film silindi. „Televizyona aktar“ ile yayından kalkar.",
        (Lang::En, FilmDeleted) => "Film deleted. “Send to the TV” takes it off air.",

        (Lang::De, FolderFailed) => "Der Ordner für den Sender konnte nicht angelegt werden.",
        (Lang::Tr, FolderFailed) => "Kanal klasörü oluşturulamadı.",
        (Lang::En, FolderFailed) => "The channel folder could not be created.",

        (Lang::De, LanguageTitle) => "Sprache",
        (Lang::Tr, LanguageTitle) => "Dil",
        (Lang::En, LanguageTitle) => "Language",

        (Lang::De, LanguageIntro) => "Gilt für den Fernseher und diese Verwaltung. Der Fernseher startet dafür kurz neu.",
        (Lang::Tr, LanguageIntro) => "Televizyon ve bu yönetim paneli için geçerlidir. Televizyon bunun için kısa süre yeniden başlar.",
        (Lang::En, LanguageIntro) => "Applies to the TV and this console. The TV restarts briefly for it.",

        (Lang::De, IptvTitle) => "Fernsehsender (IPTV)",
        (Lang::Tr, IptvTitle) => "TV kanalları (IPTV)",
        (Lang::En, IptvTitle) => "TV channels (IPTV)",

        (Lang::De, IptvIntro) => "Die Adresse der Senderliste bekommen Sie von Ihrem IPTV-Anbieter. Der Fernseher startet nach dem Speichern kurz neu.",
        (Lang::Tr, IptvIntro) => "Kanal listesinin adresini IPTV sağlayıcınızdan alırsınız. Kaydettikten sonra televizyon kısa süre yeniden başlar.",
        (Lang::En, IptvIntro) => "Your IPTV provider gives you the channel list address. The TV restarts briefly after saving.",

        (Lang::De, IptvCurrent) => "Eingerichtet bei:",
        (Lang::Tr, IptvCurrent) => "Kayıtlı sağlayıcı:",
        (Lang::En, IptvCurrent) => "Set up with:",

        (Lang::De, IptvNone) => "Noch keine Senderliste eingerichtet.",
        (Lang::Tr, IptvNone) => "Henüz kanal listesi ayarlanmadı.",
        (Lang::En, IptvNone) => "No channel list set up yet.",

        (Lang::De, IptvM3u) => "Adresse der Senderliste (M3U)",
        (Lang::Tr, IptvM3u) => "Kanal listesi adresi (M3U)",
        (Lang::En, IptvM3u) => "Channel list address (M3U)",

        (Lang::De, IptvEpg) => "Adresse der Programmzeitschrift (EPG, optional)",
        (Lang::Tr, IptvEpg) => "Yayın akışı adresi (EPG, isteğe bağlı)",
        (Lang::En, IptvEpg) => "Programme guide address (EPG, optional)",

        (Lang::De, IptvInvalid) => "Bitte eine Adresse eingeben, die mit http:// oder https:// beginnt, ohne Leerzeichen.",
        (Lang::Tr, IptvInvalid) => "Lütfen http:// veya https:// ile başlayan, boşluk içermeyen bir adres girin.",
        (Lang::En, IptvInvalid) => "Please enter an address starting with http:// or https://, without spaces.",
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
