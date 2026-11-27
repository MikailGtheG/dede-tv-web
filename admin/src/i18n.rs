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
    RemoteTitle,
    RemoteIntro,
    RemoteHome,
    RemoteAway,
    WgDevices,
    WgNone,
    WgAddTitle,
    WgName,
    WgEndpoint,
    WgRouterHint,
    WgAdd,
    WgRemove,
    WgInvalid,
    WgPeerTitle,
    WgPeerHint,
    WgPeerDone,
    WgPeerReady,
    TsIntro,
    TsOff,
    TsWaiting,
    TsOn,
    TsConnect,
    TsCheck,
    TsDisconnect,
    TsLoginTitle,
    TsLoginHint,
    TsOpen,
    SystemTitle,
    SystemIntro,
    RestartTv,
    RestartTvHint,
    Reboot,
    RebootHint,
    DiagTitle,
    DiagHint,
    DiagRun,
    DiagUptime,
    DiagHours,
    DiagMinutes,
    DiagMedia,
    DiagFreeOf,
    DiagSystem,
    DiagFailed,
    DiagNone,
    UnitTv,
    UnitGuard,
    UnitConsole,
    StateActive,
    StateInactive,
    StateFailed,
    StateOther,
    DiagDone,
    UpdTitle,
    UpdIntro,
    UpdCurrent,
    UpdAvailable,
    UpdUpToDate,
    UpdChecking,
    UpdDownloading,
    UpdRestarting,
    UpdFailed,
    UpdErrUnreachable,
    UpdErrBadImage,
    UpdErrNoSpace,
    UpdErrNoPartition,
    UpdErrDidNotStart,
    UpdErrOther,
    UpdLastCheck,
    UpdCheck,
    UpdInstall,
    UpdInstallHint,
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

        (Lang::De, RemoteTitle) => "Fernzugriff",
        (Lang::Tr, RemoteTitle) => "Uzaktan erişim",
        (Lang::En, RemoteTitle) => "Remote access",

        (Lang::De, RemoteIntro) => "Im Heimnetz ist diese Verwaltung direkt erreichbar. Von unterwegs geht es über Tailscale (am einfachsten) oder über WireGuard, wo jedes Handy und jeder Laptop einen eigenen Zugang bekommt.",
        (Lang::Tr, RemoteIntro) => "Bu yönetim paneline ev ağından doğrudan ulaşılır. Dışarıdan Tailscale (en kolayı) veya her telefonun ve dizüstü bilgisayarın kendi erişimini aldığı WireGuard ile bağlanılır.",
        (Lang::En, RemoteIntro) => "At home this console is reachable directly. From outside use Tailscale (easiest) or WireGuard, where every phone and laptop gets its own access.",

        (Lang::De, RemoteHome) => "Im Heimnetz:",
        (Lang::Tr, RemoteHome) => "Ev ağında:",
        (Lang::En, RemoteHome) => "At home:",

        (Lang::De, RemoteAway) => "Unterwegs über WireGuard:",
        (Lang::Tr, RemoteAway) => "Dışarıdan WireGuard ile:",
        (Lang::En, RemoteAway) => "Away via WireGuard:",

        (Lang::De, WgDevices) => "Geräte mit Zugang",
        (Lang::Tr, WgDevices) => "Erişimi olan cihazlar",
        (Lang::En, WgDevices) => "Devices with access",

        (Lang::De, WgNone) => "Noch kein Gerät eingerichtet.",
        (Lang::Tr, WgNone) => "Henüz cihaz eklenmedi.",
        (Lang::En, WgNone) => "No device set up yet.",

        (Lang::De, WgAddTitle) => "Neues Gerät hinzufügen",
        (Lang::Tr, WgAddTitle) => "Yeni cihaz ekle",
        (Lang::En, WgAddTitle) => "Add a device",

        (Lang::De, WgName) => "Name des Geräts, z. B. Handy Mikail",
        (Lang::Tr, WgName) => "Cihazın adı, örn. Mikail'in telefonu",
        (Lang::En, WgName) => "Device name, e.g. Mikail's phone",

        (Lang::De, WgEndpoint) => "Adresse Ihres Internetanschlusses (DynDNS-Name oder feste IP, auf Wunsch mit :Port)",
        (Lang::Tr, WgEndpoint) => "İnternet bağlantınızın adresi (DynDNS adı veya sabit IP, isterseniz :port ile)",
        (Lang::En, WgEndpoint) => "Address of your internet connection (DynDNS name or fixed IP, optionally with :port)",

        (Lang::De, WgRouterHint) => "Am Router muss der UDP-Port 51820 an diesen Fernseher weitergeleitet sein.",
        (Lang::Tr, WgRouterHint) => "Modemde UDP port 51820 bu televizyona yönlendirilmiş olmalıdır.",
        (Lang::En, WgRouterHint) => "Your router must forward UDP port 51820 to this TV.",

        (Lang::De, WgAdd) => "Zugang erstellen",
        (Lang::Tr, WgAdd) => "Erişim oluştur",
        (Lang::En, WgAdd) => "Create access",

        (Lang::De, WgRemove) => "Entfernen",
        (Lang::Tr, WgRemove) => "Kaldır",
        (Lang::En, WgRemove) => "Remove",

        (Lang::De, WgInvalid) => "Bitte einen Namen (bis 32 Zeichen, ohne Sonderzeichen) und eine gültige Adresse eingeben.",
        (Lang::Tr, WgInvalid) => "Lütfen bir ad (en fazla 32 karakter, özel karakter olmadan) ve geçerli bir adres girin.",
        (Lang::En, WgInvalid) => "Please enter a name (up to 32 characters, no special characters) and a valid address.",

        (Lang::De, WgPeerTitle) => "Zugang für das neue Gerät",
        (Lang::Tr, WgPeerTitle) => "Yeni cihazın erişimi",
        (Lang::En, WgPeerTitle) => "Access for the new device",

        (Lang::De, WgPeerHint) => "In der WireGuard-App „Tunnel hinzufügen“ und dann „QR-Code scannen“ wählen. Dieser Zugang wird nur jetzt angezeigt.",
        (Lang::Tr, WgPeerHint) => "WireGuard uygulamasında „Tünel ekle“ ve ardından „QR kodu tara“ seçin. Bu erişim yalnızca şimdi gösterilir.",
        (Lang::En, WgPeerHint) => "In the WireGuard app choose “Add tunnel”, then “Scan QR code”. This access is shown only now.",

        (Lang::De, WgPeerDone) => "Fertig, Zugang ausblenden",
        (Lang::Tr, WgPeerDone) => "Tamam, erişimi gizle",
        (Lang::En, WgPeerDone) => "Done, hide the access",

        (Lang::De, WgPeerReady) => "Der Zugang ist eingerichtet.",
        (Lang::Tr, WgPeerReady) => "Erişim oluşturuldu.",
        (Lang::En, WgPeerReady) => "The access is set up.",

        (Lang::De, TsIntro) => "Mit Tailscale erreichen Sie die Verwaltung von überall, ohne Einstellungen am Router. Dafür brauchen Sie ein kostenloses Tailscale-Konto.",
        (Lang::Tr, TsIntro) => "Tailscale ile yönetim paneline modemde ayar yapmadan her yerden ulaşırsınız. Bunun için ücretsiz bir Tailscale hesabı gerekir.",
        (Lang::En, TsIntro) => "With Tailscale you reach the console from anywhere without router settings. You need a free Tailscale account.",

        (Lang::De, TsOff) => "Nicht verbunden.",
        (Lang::Tr, TsOff) => "Bağlı değil.",
        (Lang::En, TsOff) => "Not connected.",

        (Lang::De, TsWaiting) => "Die Anmeldung ist noch nicht abgeschlossen.",
        (Lang::Tr, TsWaiting) => "Giriş henüz tamamlanmadı.",
        (Lang::En, TsWaiting) => "Sign-in is not finished yet.",

        (Lang::De, TsOn) => "Verbunden. Unterwegs erreichbar unter:",
        (Lang::Tr, TsOn) => "Bağlı. Dışarıdan şu adresten ulaşılır:",
        (Lang::En, TsOn) => "Connected. Reachable from anywhere at:",

        (Lang::De, TsConnect) => "Mit Tailscale verbinden",
        (Lang::Tr, TsConnect) => "Tailscale ile bağlan",
        (Lang::En, TsConnect) => "Connect with Tailscale",

        (Lang::De, TsCheck) => "Status prüfen",
        (Lang::Tr, TsCheck) => "Durumu kontrol et",
        (Lang::En, TsCheck) => "Check status",

        (Lang::De, TsDisconnect) => "Tailscale trennen",
        (Lang::Tr, TsDisconnect) => "Tailscale bağlantısını kes",
        (Lang::En, TsDisconnect) => "Disconnect Tailscale",

        (Lang::De, TsLoginTitle) => "Bei Tailscale anmelden",
        (Lang::Tr, TsLoginTitle) => "Tailscale'e giriş yapın",
        (Lang::En, TsLoginTitle) => "Sign in to Tailscale",

        (Lang::De, TsLoginHint) => "Öffnen Sie den Link oder scannen Sie den QR-Code mit dem Handy und melden Sie sich an. Tippen Sie danach auf der Seite Fernzugriff auf „Status prüfen“.",
        (Lang::Tr, TsLoginHint) => "Bağlantıyı açın veya QR kodunu telefonla tarayın ve giriş yapın. Ardından Uzaktan erişim sayfasında „Durumu kontrol et“e dokunun.",
        (Lang::En, TsLoginHint) => "Open the link or scan the QR code with your phone and sign in. Then tap “Check status” on the remote access page.",

        (Lang::De, TsOpen) => "Anmeldeseite öffnen",
        (Lang::Tr, TsOpen) => "Giriş sayfasını aç",
        (Lang::En, TsOpen) => "Open the sign-in page",

        (Lang::De, SystemTitle) => "Fernseher und Diagnose",
        (Lang::Tr, SystemTitle) => "Televizyon ve tanılama",
        (Lang::En, SystemTitle) => "TV and diagnostics",

        (Lang::De, SystemIntro) => "Wenn etwas nicht stimmt: zuerst den Fernseher neu starten. Hilft das nicht, das ganze Gerät.",
        (Lang::Tr, SystemIntro) => "Bir sorun varsa: önce televizyonu yeniden başlatın. İşe yaramazsa tüm cihazı.",
        (Lang::En, SystemIntro) => "If something is wrong: restart the TV first. If that does not help, the whole device.",

        (Lang::De, RestartTv) => "Fernseher neu starten",
        (Lang::Tr, RestartTv) => "Televizyonu yeniden başlat",
        (Lang::En, RestartTv) => "Restart the TV",

        (Lang::De, RestartTvHint) => "Das Bild ist etwa eine halbe Minute weg, danach läuft der letzte Sender wieder.",
        (Lang::Tr, RestartTvHint) => "Görüntü yaklaşık yarım dakika gider, sonra son kanal tekrar açılır.",
        (Lang::En, RestartTvHint) => "The picture is gone for about half a minute, then the last channel plays again.",

        (Lang::De, Reboot) => "Gerät neu starten",
        (Lang::Tr, Reboot) => "Cihazı yeniden başlat",
        (Lang::En, Reboot) => "Restart the device",

        (Lang::De, RebootHint) => "Dauert etwa zwei Minuten. So lange ist diese Seite nicht erreichbar.",
        (Lang::Tr, RebootHint) => "Yaklaşık iki dakika sürer. Bu süre boyunca bu sayfaya ulaşılamaz.",
        (Lang::En, RebootHint) => "Takes about two minutes. This page is unreachable meanwhile.",

        (Lang::De, DiagTitle) => "Diagnose",
        (Lang::Tr, DiagTitle) => "Tanılama",
        (Lang::En, DiagTitle) => "Diagnostics",

        (Lang::De, DiagHint) => "Zeigt, ob alle Teile laufen. Hilfreich, wenn Sie jemanden um Hilfe bitten.",
        (Lang::Tr, DiagHint) => "Tüm parçaların çalışıp çalışmadığını gösterir. Birinden yardım isterken işe yarar.",
        (Lang::En, DiagHint) => "Shows whether all parts are running. Useful when you ask someone for help.",

        (Lang::De, DiagRun) => "Diagnose erstellen",
        (Lang::Tr, DiagRun) => "Tanılama oluştur",
        (Lang::En, DiagRun) => "Run diagnostics",

        (Lang::De, DiagUptime) => "Läuft seit",
        (Lang::Tr, DiagUptime) => "Çalışma süresi",
        (Lang::En, DiagUptime) => "Running for",

        (Lang::De, DiagHours) => "Std.",
        (Lang::Tr, DiagHours) => "sa",
        (Lang::En, DiagHours) => "h",

        (Lang::De, DiagMinutes) => "Min.",
        (Lang::Tr, DiagMinutes) => "dk",
        (Lang::En, DiagMinutes) => "min",

        (Lang::De, DiagMedia) => "Speicher für Filme",
        (Lang::Tr, DiagMedia) => "Filmler için alan",
        (Lang::En, DiagMedia) => "Storage for films",

        (Lang::De, DiagFreeOf) => "frei von",
        (Lang::Tr, DiagFreeOf) => "boş, toplam",
        (Lang::En, DiagFreeOf) => "free of",

        (Lang::De, DiagSystem) => "System",
        (Lang::Tr, DiagSystem) => "Sistem",
        (Lang::En, DiagSystem) => "System",

        (Lang::De, DiagFailed) => "Dienste mit Fehlern",
        (Lang::Tr, DiagFailed) => "Hatalı servisler",
        (Lang::En, DiagFailed) => "Services with errors",

        (Lang::De, DiagNone) => "keine",
        (Lang::Tr, DiagNone) => "yok",
        (Lang::En, DiagNone) => "none",

        (Lang::De, UnitTv) => "Fernsehbild",
        (Lang::Tr, UnitTv) => "Televizyon görüntüsü",
        (Lang::En, UnitTv) => "TV picture",

        (Lang::De, UnitGuard) => "Senderwächter",
        (Lang::Tr, UnitGuard) => "Kanal bekçisi",
        (Lang::En, UnitGuard) => "Channel guard",

        (Lang::De, UnitConsole) => "Verwaltung",
        (Lang::Tr, UnitConsole) => "Yönetim paneli",
        (Lang::En, UnitConsole) => "Console",

        (Lang::De, StateActive) => "läuft",
        (Lang::Tr, StateActive) => "çalışıyor",
        (Lang::En, StateActive) => "running",

        (Lang::De, StateInactive) => "aus",
        (Lang::Tr, StateInactive) => "kapalı",
        (Lang::En, StateInactive) => "off",

        (Lang::De, StateFailed) => "Fehler",
        (Lang::Tr, StateFailed) => "hata",
        (Lang::En, StateFailed) => "error",

        (Lang::De, StateOther) => "unbekannt",
        (Lang::Tr, StateOther) => "bilinmiyor",
        (Lang::En, StateOther) => "unknown",

        (Lang::De, DiagDone) => "Stand: gerade eben.",
        (Lang::Tr, DiagDone) => "Durum: şu an.",
        (Lang::En, DiagDone) => "Status: just now.",

        (Lang::De, UpdTitle) => "Updates",
        (Lang::Tr, UpdTitle) => "Güncellemeler",
        (Lang::En, UpdTitle) => "Updates",

        (Lang::De, UpdIntro) => "Neue Versionen werden nur mit gültiger Signatur installiert. Startet eine neue Version nicht richtig, kehrt der Fernseher von selbst zur vorherigen zurück. Ihre Sender, Filme und Einstellungen bleiben erhalten.",
        (Lang::Tr, UpdIntro) => "Yeni sürümler yalnızca geçerli imzayla yüklenir. Yeni sürüm düzgün başlamazsa televizyon kendiliğinden önceki sürüme döner. Kanallarınız, filmleriniz ve ayarlarınız korunur.",
        (Lang::En, UpdIntro) => "New versions are only installed with a valid signature. If a new version does not start properly, the TV returns to the previous one by itself. Your channels, films and settings are kept.",

        (Lang::De, UpdCurrent) => "Installierte Version:",
        (Lang::Tr, UpdCurrent) => "Yüklü sürüm:",
        (Lang::En, UpdCurrent) => "Installed version:",

        (Lang::De, UpdAvailable) => "Neue Version verfügbar:",
        (Lang::Tr, UpdAvailable) => "Yeni sürüm mevcut:",
        (Lang::En, UpdAvailable) => "New version available:",

        (Lang::De, UpdUpToDate) => "Der Fernseher ist auf dem neuesten Stand.",
        (Lang::Tr, UpdUpToDate) => "Televizyon güncel.",
        (Lang::En, UpdUpToDate) => "The TV is up to date.",

        (Lang::De, UpdChecking) => "Suche nach Updates …",
        (Lang::Tr, UpdChecking) => "Güncellemeler aranıyor …",
        (Lang::En, UpdChecking) => "Looking for updates …",

        (Lang::De, UpdDownloading) => "Das Update wird geladen:",
        (Lang::Tr, UpdDownloading) => "Güncelleme indiriliyor:",
        (Lang::En, UpdDownloading) => "Downloading the update:",

        (Lang::De, UpdRestarting) => "Das Update ist geladen. Der Fernseher startet jetzt neu und probiert die neue Version aus.",
        (Lang::Tr, UpdRestarting) => "Güncelleme indirildi. Televizyon şimdi yeniden başlıyor ve yeni sürümü deniyor.",
        (Lang::En, UpdRestarting) => "The update is downloaded. The TV now restarts and tries the new version.",

        (Lang::De, UpdFailed) => "Das hat nicht geklappt:",
        (Lang::Tr, UpdFailed) => "Bu işe yaramadı:",
        (Lang::En, UpdFailed) => "That did not work:",

        (Lang::De, UpdErrUnreachable) => "Keine Verbindung zum Update-Server.",
        (Lang::Tr, UpdErrUnreachable) => "Güncelleme sunucusuna bağlantı yok.",
        (Lang::En, UpdErrUnreachable) => "No connection to the update server.",

        (Lang::De, UpdErrBadImage) => "Die geladene Datei war beschädigt. Bitte später noch einmal versuchen.",
        (Lang::Tr, UpdErrBadImage) => "İndirilen dosya bozuktu. Lütfen daha sonra tekrar deneyin.",
        (Lang::En, UpdErrBadImage) => "The downloaded file was damaged. Please try again later.",

        (Lang::De, UpdErrNoSpace) => "Nicht genug Platz für das Update.",
        (Lang::Tr, UpdErrNoSpace) => "Güncelleme için yeterli yer yok.",
        (Lang::En, UpdErrNoSpace) => "Not enough space for the update.",

        (Lang::De, UpdErrNoPartition) => "Auf diesem Gerät ist kein Platz für Updates eingerichtet. Dafür muss es einmal neu installiert werden.",
        (Lang::Tr, UpdErrNoPartition) => "Bu cihazda güncellemeler için yer ayrılmamış. Bunun için bir kez yeniden kurulması gerekir.",
        (Lang::En, UpdErrNoPartition) => "This device has no space set up for updates. It needs to be installed once more for that.",

        (Lang::De, UpdErrDidNotStart) => "Die neue Version ist nicht richtig gestartet. Der Fernseher läuft wieder mit der vorherigen Version.",
        (Lang::Tr, UpdErrDidNotStart) => "Yeni sürüm düzgün başlamadı. Televizyon yine önceki sürümle çalışıyor.",
        (Lang::En, UpdErrDidNotStart) => "The new version did not start properly. The TV runs the previous version again.",

        (Lang::De, UpdErrOther) => "Unbekannter Fehler.",
        (Lang::Tr, UpdErrOther) => "Bilinmeyen hata.",
        (Lang::En, UpdErrOther) => "Unknown error.",

        (Lang::De, UpdLastCheck) => "Zuletzt geprüft (UTC):",
        (Lang::Tr, UpdLastCheck) => "Son kontrol (UTC):",
        (Lang::En, UpdLastCheck) => "Last checked (UTC):",

        (Lang::De, UpdCheck) => "Nach Updates suchen",
        (Lang::Tr, UpdCheck) => "Güncellemeleri ara",
        (Lang::En, UpdCheck) => "Check for updates",

        (Lang::De, UpdInstall) => "Update installieren",
        (Lang::Tr, UpdInstall) => "Güncellemeyi yükle",
        (Lang::En, UpdInstall) => "Install the update",

        (Lang::De, UpdInstallHint) => "Dauert einige Minuten. Danach startet der Fernseher einmal neu.",
        (Lang::Tr, UpdInstallHint) => "Birkaç dakika sürer. Ardından televizyon bir kez yeniden başlar.",
        (Lang::En, UpdInstallHint) => "Takes a few minutes. Then the TV restarts once.",
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
