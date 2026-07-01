use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::Deserialize;

/// A simple weather CLI that fetches current weather for a given city.
#[derive(Parser)]
#[command(name = "weather-cli")]
#[command(about = "Fetch current weather for a city using Open-Meteo (no API key required)")]
struct Cli {
    /// Name of the city (e.g., "London", "New York", "Tokyo")
    city: String,
}

// --- Geocoding API types ---

#[derive(Deserialize, Debug)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Deserialize, Debug)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    country_code: Option<String>,
}

// --- Weather API types ---

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current_weather: CurrentWeather,
}

#[derive(Deserialize, Debug)]
struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    weathercode: i64,
    time: String,
}

/// Convert a WMO weather code into a human-readable description and emoji.
fn weather_description(code: i64) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear sky", "☀️"),
        1 | 2 | 3 => ("Partly cloudy", "⛅"),
        45 | 48 => ("Foggy", "🌫️"),
        51 | 53 | 55 => ("Drizzle", "🌦️"),
        56 | 57 => ("Freezing drizzle", "🌧️"),
        61 | 63 | 65 => ("Rain", "🌧️"),
        66 | 67 => ("Freezing rain", "🌧️"),
        71 | 73 | 75 => ("Snowfall", "❄️"),
        77 => ("Snow grains", "❄️"),
        80 | 81 | 82 => ("Rain showers", "🌦️"),
        85 | 86 => ("Snow showers", "🌨️"),
        95 => ("Thunderstorm", "⛈️"),
        96 | 99 => ("Thunderstorm with hail", "⛈️"),
        _ => ("Unknown", "❓"),
    }
}

fn geocode_city(city: &str) -> Result<GeocodingResult> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding(city)
    );

    let resp = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to reach geocoding API for city '{}'", city))?;

    if !resp.status().is_success() {
        return Err(anyhow!(
            "Geocoding API returned HTTP {} for city '{}'",
            resp.status(),
            city
        ));
    }

    let geo: GeocodingResponse = resp
        .json()
        .context("Failed to parse geocoding API response")?;

    geo.results
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| anyhow!("Could not find city '{}'", city))
}

fn fetch_weather(lat: f64, lon: f64) -> Result<WeatherResponse> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
        lat, lon
    );

    let resp = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to reach weather API for coordinates ({}, {})", lat, lon))?;

    if !resp.status().is_success() {
        return Err(anyhow!(
            "Weather API returned HTTP {} for coordinates ({}, {})",
            resp.status(),
            lat,
            lon
        ));
    }

    let weather: WeatherResponse = resp
        .json()
        .context("Failed to parse weather API response")?;

    Ok(weather)
}

/// Minimal percent-encoding: only spaces -> %20, leaving ASCII alone.
fn urlencoding(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => {
                c.to_string()
            }
            c => {
                let bytes = c.to_string().into_bytes();
                bytes
                    .iter()
                    .map(|&b| format!("%{:02X}", b))
                    .collect::<String>()
            }
        })
        .collect()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🔍 Looking up city: \"{}\"...", cli.city);

    let location = geocode_city(&cli.city)?;
    let location_name = match (&location.country, &location.country_code) {
        (Some(country), Some(code)) => format!("{}, {} ({})", location.name, country, code),
        (Some(country), _) => format!("{}, {}", location.name, country),
        _ => location.name.clone(),
    };

    println!("📍 Location: {}", location_name);
    println!(
        "   Coordinates: {:.4}, {:.4}",
        location.latitude, location.longitude
    );
    println!();

    println!("🌤️  Fetching current weather...");
    let weather = fetch_weather(location.latitude, location.longitude)?;

    let (desc, emoji) = weather_description(weather.current_weather.weathercode);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {}  Current Weather for {}", emoji, location.name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Time:       {}", weather.current_weather.time);
    println!(
        "  Condition:  {} ({}) — {}",
        desc, weather.current_weather.weathercode, emoji
    );
    println!(
        "  🌡️  Temperature: {:.1}°C",
        weather.current_weather.temperature
    );
    println!(
        "  💨 Wind Speed:  {:.1} km/h",
        weather.current_weather.windspeed
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
