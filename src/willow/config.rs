use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum WillowAudioCodec {
    AmrWb,
    Pcm,
}

#[derive(Deserialize, Serialize)]
enum WillowAudioResponseType {
    Chimes,
    None,
    #[serde(rename = "TTS")]
    Tts,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum WillowCommandEndpoint {
    #[serde(rename = "Home Assistant")]
    HomeAssistant,
    #[serde(rename = "openHAB")]
    OpenHab,
    Mqtt,
    Rest,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum WillowMqttAuthType {
    None,
    UserPw,
}

#[derive(Deserialize, Serialize)]
enum WillowNtpConfig {
    #[serde(rename = "DHCP")]
    Dhcp,
    Host,
}

#[derive(Deserialize, Serialize)]
enum WillowRestAuthType {
    #[serde(rename = "None")]
    NoneType,
    Basic,
    Header,
}

#[derive(Deserialize, Serialize)]
pub enum WillowSpeechRecMode {
    #[serde(rename = "WIS")]
    Wis,
}

#[derive(Deserialize, Serialize)]
pub enum WillowWakeMode {
    #[serde(rename = "1CH_90")]
    _1Ch90,
    #[serde(rename = "1CH_95")]
    _1Ch95,
    #[serde(rename = "2CH_90")]
    _2Ch90,
    #[serde(rename = "2CH_95")]
    _2Ch95,
    #[serde(rename = "3CH_90")]
    _3Ch90,
    #[serde(rename = "3CH_95")]
    _3Ch95,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WillowWakeWord {
    Alexa,
    Hiesp,
    Hilexin,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Serialize)]
pub struct WillowConfig {
    aec: bool,
    audio_codec: WillowAudioCodec,
    audio_response_type: WillowAudioResponseType,
    bss: bool,
    command_endpoint: WillowCommandEndpoint,
    display_timeout: u32,
    hass_host: Option<String>,
    hass_port: Option<u16>,
    hass_tls: Option<bool>,
    hass_token: Option<String>,
    lcd_brightness: u32,
    mic_gain: u8,
    mqtt_auth_type: Option<WillowMqttAuthType>,
    mqtt_host: Option<String>,
    mqtt_password: Option<String>,
    mqtt_port: Option<String>,
    mqtt_tls: Option<bool>,
    mqtt_topic: Option<String>,
    mqtt_username: Option<String>,
    multiwake: bool,
    ntp_config: WillowNtpConfig,
    ntp_host: Option<String>,
    openhab_token: Option<String>,
    openhab_url: Option<String>,
    record_buffer: u8,
    rest_auth_header: Option<String>,
    rest_auth_pass: Option<String>,
    rest_auth_type: Option<WillowRestAuthType>,
    rest_auth_user: Option<String>,
    rest_url: Option<String>,
    show_prereleases: bool,
    speaker_volume: u8,
    speech_rec_mode: Option<WillowSpeechRecMode>,
    stream_timeout: u8,
    timezone: String,
    timezone_name: String,
    vad_mode: u8,
    vad_timeout: u32,
    wake_confirmation: bool,
    wake_mode: WillowWakeMode,
    wake_word: WillowWakeWord,
    was_mode: bool,
    wis_tts_url: Option<String>,
    wis_tts_url_v2: Option<String>,
    wis_url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct WillowNvsWas {
    url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct WillowNvsWifi {
    psk: String,
    ssid: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct WillowNvsConfig {
    pub was: WillowNvsWas,
    pub wifi: WillowNvsWifi,
}
