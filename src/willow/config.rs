use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;

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
#[skip_serializing_none]
pub struct WillowConfig {
    #[serde(deserialize_with = "deserialize_string_to_bool")]
    aec: bool,
    audio_codec: WillowAudioCodec,
    audio_response_type: WillowAudioResponseType,
    #[serde(deserialize_with = "deserialize_string_to_bool")]
    bss: bool,
    command_endpoint: WillowCommandEndpoint,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    display_timeout: u32,
    hass_host: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_to_option_number"
    )]
    hass_port: Option<u16>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_to_option_bool"
    )]
    hass_tls: Option<bool>,
    hass_token: Option<String>,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    lcd_brightness: u32,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    mic_gain: u8,
    mqtt_auth_type: Option<WillowMqttAuthType>,
    mqtt_host: Option<String>,
    mqtt_password: Option<String>,
    mqtt_port: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_to_option_bool"
    )]
    mqtt_tls: Option<bool>,
    mqtt_topic: Option<String>,
    mqtt_username: Option<String>,
    #[serde(deserialize_with = "deserialize_string_to_bool")]
    multiwake: bool,
    ntp_config: WillowNtpConfig,
    ntp_host: Option<String>,
    openhab_token: Option<String>,
    openhab_url: Option<String>,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    record_buffer: u8,
    rest_auth_header: Option<String>,
    rest_auth_pass: Option<String>,
    rest_auth_type: Option<WillowRestAuthType>,
    rest_auth_user: Option<String>,
    rest_url: Option<String>,
    #[serde(deserialize_with = "deserialize_string_to_bool")]
    show_prereleases: bool,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    speaker_volume: u8,
    speech_rec_mode: Option<WillowSpeechRecMode>,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    stream_timeout: u8,
    timezone: String,
    timezone_name: String,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    vad_mode: u8,
    #[serde(deserialize_with = "deserialize_string_to_number")]
    vad_timeout: u32,
    #[serde(deserialize_with = "deserialize_string_to_bool")]
    wake_confirmation: bool,
    wake_mode: WillowWakeMode,
    wake_word: WillowWakeWord,
    #[serde(deserialize_with = "deserialize_string_to_bool")]
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

fn deserialize_optional_string_to_option_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s = Option::<String>::deserialize(deserializer)?;
    match opt_s {
        None => Ok(None),
        Some(s) => match s.to_lowercase().as_str() {
            "false" => Ok(Some(false)),
            "true" => Ok(Some(true)),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"true or false",
            )),
        },
    }
}

fn deserialize_optional_string_to_option_number<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let opt_s = Option::<String>::deserialize(deserializer)?;
    match opt_s {
        None => Ok(None),
        Some(s) => T::from_str(&s).map(Some).map_err(serde::de::Error::custom),
    }
}

fn deserialize_string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&s),
            &"true or false",
        )),
    }
}

fn deserialize_string_to_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
