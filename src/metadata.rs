extern crate dbus;

use std::collections::HashMap;
use std::time::Duration;

use dbus::arg::{cast, RefArg, Variant};

use super::DBusError;

/// A structured representation of the `Player` metadata.
///
/// * [Read more about the MPRIS2 `Metadata_Map`
/// type.](https://specifications.freedesktop.org/mpris-spec/latest/Track_List_Interface.html#Mapping:Metadata_Map)
/// * [Read MPRIS v2 metadata guidelines](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/)
#[derive(Debug)]
pub struct Metadata {
    track_id: String,
    album_artists: Option<Vec<String>>,
    album_name: Option<String>,
    art_url: Option<String>,
    artists: Option<Vec<String>>,
    auto_rating: Option<f64>,
    disc_number: Option<i32>,
    length_in_microseconds: Option<u64>,
    title: Option<String>,
    track_number: Option<i32>,
    url: Option<String>,
    rest: HashMap<String, Variant<Box<RefArg>>>,
}

/// Holds a dynamically-typed metadata value.
///
/// You will need to type-check this at runtime in order to use the value.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// Value is a string.
    String(String),

    /// Value is a 64-bit integer.
    I64(i64),

    /// Value is a 32-bit integer.
    I32(i32),

    /// Value is a 64-bit float.
    F64(f64),

    /// Value is a boolean.
    Bool(bool),

    /// Unsupported value type.
    Unsupported,
}

/// Simpler enum to encode the type of a `MetadataValue`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ValueKind {
    /// Value is a string.
    String,

    /// Value is a 64-bit integer.
    I64,

    /// Value is a 32-bit integer.
    I32,

    /// Value is a 64-bit float.
    F64,

    /// Value is a boolean.
    Bool,

    /// Unsupported value type.
    Unsupported,
}

impl Metadata {
    /// Create a new `Metadata` struct with a given `track_id`.
    ///
    /// This is mostly useful for test fixtures and other places where you want to work with mock
    /// data.
    pub fn new(track_id: String) -> Self {
        let mut builder = MetadataBuilder::new();
        builder.track_id = Some(track_id);
        builder.finish().unwrap()
    }

    pub(crate) fn new_from_dbus(
        metadata: HashMap<String, Variant<Box<RefArg>>>,
    ) -> Result<Metadata, DBusError> {
        MetadataBuilder::build_from_metadata(metadata)
    }

    /// The track ID.
    ///
    /// Based on `mpris:trackId`
    /// > A unique identity for this track within the context of an MPRIS object.
    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    /// A list of artists of the album the track appears on.
    ///
    /// Based on `xesam:albumArtist`
    /// > The album artist(s).
    pub fn album_artists(&self) -> Option<&Vec<String>> {
        self.album_artists.as_ref()
    }

    /// The name of the album the track appears on.
    ///
    /// Based on `xesam:album`
    /// > The album name.
    pub fn album_name(&self) -> Option<&str> {
        self.album_name.as_ref().map(String::as_ref)
    }

    /// An URL to album art of the current track.
    ///
    /// Based on `mpris:artUrl`
    /// > The location of an image representing the track or album. Clients should not assume this
    /// > will continue to exist when the media player stops giving out the URL.
    pub fn art_url(&self) -> Option<&str> {
        self.art_url.as_ref().map(String::as_ref)
    }

    /// A list of artists of the track.
    ///
    /// Based on `xesam:artist`
    /// > The track artist(s).
    pub fn artists(&self) -> Option<&Vec<String>> {
        self.artists.as_ref()
    }

    /// Based on `xesam:autoRating`
    /// > An automatically-generated rating, based on things such as how often it has been played.
    /// > This should be in the range 0.0 to 1.0.
    pub fn auto_rating(&self) -> Option<f64> {
        self.auto_rating
    }

    /// Based on `xesam:discNumber`
    /// > The disc number on the album that this track is from.
    pub fn disc_number(&self) -> Option<i32> {
        self.disc_number
    }

    /// The duration of the track, in microseconds
    ///
    /// Based on `mpris:length`
    /// > The duration of the track in microseconds.
    pub fn length_in_microseconds(&self) -> Option<u64> {
        self.length_in_microseconds
    }

    /// The duration of the track, as a `Duration`
    ///
    /// Based on `mpris:length`.
    pub fn length(&self) -> Option<Duration> {
        use extensions::DurationExtensions;
        self.length_in_microseconds
            .clone()
            .map(Duration::from_micros_ext)
    }

    /// The name of the track.
    ///
    /// Based on `xesam:title`
    /// > The track title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(String::as_str)
    }

    /// The track number on the disc of the album the track appears on.
    ///
    /// Based on `xesam:trackNumber`
    /// > The track number on the album disc.
    pub fn track_number(&self) -> Option<i32> {
        self.track_number
    }

    /// A URL to the media being played.
    ///
    /// Based on `xesam:url`
    /// > The location of the media file.
    pub fn url(&self) -> Option<&str> {
        self.url.as_ref().map(String::as_str)
    }

    /// Remaining metadata that has not been parsed into one of the other fields of the `Metadata`,
    /// if any.
    ///
    /// **NOTE:** This method is deprecated and will be removed in version 2.0. See `rest_hash` for
    /// a successor.
    ///
    /// As an example, if the media player exposed `xesam:composer`, then you could read that
    /// String like this:
    ///
    /// ```rust
    /// # extern crate mpris;
    /// # extern crate dbus;
    /// # use mpris::Metadata;
    /// # fn main() {
    /// # let metadata = Metadata::new(String::from("1234"));
    /// use dbus::arg::RefArg;
    /// if let Some(name) = metadata.rest().get("xesam:composer").and_then(|v| v.as_str()) {
    ///     println!("Composed by: {}", name)
    /// }
    /// # }
    /// ```
    pub fn rest(&self) -> &HashMap<String, Variant<Box<RefArg>>> {
        &self.rest
    }

    /// Remaining metadata that has not been parsed into one of the other fields of the `Metadata`,
    /// if any.
    ///
    /// **NOTE:** This method will be renamed and reworked in version 2.0 in order to replace
    /// `rest`. Note that this method will likely become cheaper at that point.
    ///
    /// **NOTE:** This method returns an *owned* value in the 1.x series for
    /// backwards-compatibility reasons. That means that this method is expensive to call and you
    /// should reuse the value if possible.
    ///
    /// As an example, if the media player exposed `xesam:composer`, then you could read that
    /// String like this:
    ///
    /// ```rust
    /// # extern crate mpris;
    /// # extern crate dbus;
    /// use mpris::{Metadata, MetadataValue};
    /// # fn main() {
    /// # let metadata = Metadata::new(String::from("1234"));
    /// let rest_hash = metadata.rest_hash();
    /// let composer = rest_hash.get("xesam:composer");
    /// match composer {
    ///     Some(&MetadataValue::String(ref name)) => println!("Composed by: {}", name),
    ///     Some(value) => println!("xesam:composer had an unexpected type: {:?}", value.kind()),
    ///     None => println!("Composer is not set"),
    /// }
    /// # }
    /// ```
    pub fn rest_hash(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        for (key, variant) in self.rest.iter() {
            if let Some(value) = Value::from_variant(variant) {
                map.insert(key.clone(), value);
            }
        }
        map
    }
}

impl Value {
    fn from_variant(variant: &Variant<Box<RefArg>>) -> Option<Value> {
        use dbus::arg::ArgType;
        let data = &variant.0;
        match data.arg_type() {
            ArgType::String => data.as_str().map(Value::from),
            ArgType::Int64 => data.as_i64().map(Value::from),
            ArgType::Int32 => cast(data).cloned().map(|i| Value::I32(i)),
            ArgType::Double => cast(data).cloned().map(|f| Value::F64(f)),
            ArgType::Boolean => cast(data).cloned().map(|b| Value::Bool(b)),
            _ => Some(Value::Unsupported),
        }
    }

    /// Returns a simple enum representing the type of value that this value holds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate mpris;
    /// # extern crate dbus;
    /// # use mpris::Metadata;
    /// # fn main() {
    /// # let metadata = Metadata::new(String::from("1234"));
    /// # let key_name = "foo";
    /// use mpris::MetadataValueKind;
    /// let rest_hash = metadata.rest_hash();
    /// if let Some(value) = rest_hash.get(key_name) {
    ///     match value.kind() {
    ///       MetadataValueKind::String => println!("{} is a string", key_name),
    ///       MetadataValueKind::I64 | MetadataValueKind::I32 => println!("{} is an integer", key_name),
    ///       MetadataValueKind::F64 => println!("{} is a float", key_name),
    ///       MetadataValueKind::Bool => println!("{} is a boolean", key_name),
    ///       MetadataValueKind::Unsupported => println!("{} is not a supported type", key_name),
    ///     }
    /// } else {
    ///     println!("Metadata does not have a {} key", key_name);
    /// }
    /// # }
    /// ```
    pub fn kind(&self) -> ValueKind {
        match *self {
            Value::String(_) => ValueKind::String,
            Value::I64(_) => ValueKind::I64,
            Value::I32(_) => ValueKind::I32,
            Value::F64(_) => ValueKind::F64,
            Value::Bool(_) => ValueKind::Bool,
            Value::Unsupported => ValueKind::Unsupported,
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Value {
        Value::String(string)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(string: &'a str) -> Value {
        Value::String(String::from(string))
    }
}

impl From<i64> for Value {
    fn from(int: i64) -> Value {
        Value::I64(int)
    }
}

#[derive(Debug, Default)]
struct MetadataBuilder {
    track_id: Option<String>,

    album_artists: Option<Vec<String>>,
    album_name: Option<String>,
    art_url: Option<String>,
    artists: Option<Vec<String>>,
    auto_rating: Option<f64>,
    disc_number: Option<i32>,
    length_in_microseconds: Option<u64>,
    title: Option<String>,
    track_number: Option<i32>,
    url: Option<String>,

    rest: HashMap<String, Variant<Box<RefArg>>>,
}

fn cast_string_vec(value: &Variant<Box<RefArg>>) -> Option<Vec<String>> {
    value
        .0
        .as_iter()
        .map(|arr| arr.flat_map(cast_string).collect())
}

fn cast_string<T: RefArg + ?Sized>(value: &T) -> Option<String> {
    value.as_str().map(String::from)
}

impl MetadataBuilder {
    fn build_from_metadata(
        metadata: HashMap<String, Variant<Box<RefArg>>>,
    ) -> Result<Metadata, DBusError> {
        let mut builder = MetadataBuilder::new();

        for (key, value) in metadata {
            match key.as_ref() {
                "mpris:trackid" => builder.track_id = cast_string(&value),
                "mpris:length" => builder.length_in_microseconds = cast(&value.0).cloned(),
                "mpris:artUrl" => builder.art_url = cast_string(&value),
                "xesam:title" => builder.title = cast_string(&value),
                "xesam:albumArtist" => builder.album_artists = cast_string_vec(&value),
                "xesam:artist" => builder.artists = cast_string_vec(&value),
                "xesam:url" => builder.url = cast_string(&value),
                "xesam:album" => builder.album_name = cast_string(&value),
                "xesam:discNumber" => builder.disc_number = cast(&value.0).cloned(),
                "xesam:trackNumber" => builder.track_number = cast(&value.0).cloned(),
                "xesam:autoRating" => builder.auto_rating = cast(&value.0).cloned(),
                _ => builder.add_rest(key, value),
            };
        }

        builder.finish()
    }

    fn new() -> Self {
        MetadataBuilder::default()
    }

    fn add_rest(&mut self, key: String, value: Variant<Box<RefArg>>) {
        self.rest.insert(key, value);
    }

    fn finish(self) -> Result<Metadata, DBusError> {
        match self.track_id {
            Some(track_id) => Ok(Metadata {
                track_id: track_id,

                album_artists: self.album_artists,
                album_name: self.album_name,
                art_url: self.art_url,
                artists: self.artists,
                auto_rating: self.auto_rating,
                disc_number: self.disc_number,
                length_in_microseconds: self.length_in_microseconds,
                title: self.title,
                track_number: self.track_number,
                url: self.url,

                rest: self.rest,
            }),
            None => Err(DBusError::new(
                "TrackId is missing from metadata; client is not conforming to MPRIS-2",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_new_metadata() {
        let metadata = Metadata::new(String::from("foo"));
        assert_eq!(metadata.track_id, "foo");
    }

    mod rest {
        use super::*;

        fn metadata_builder() -> MetadataBuilder {
            let mut builder = MetadataBuilder::new();
            builder.track_id = Some(String::new());
            builder
        }

        fn metadata_with_rest<S>(key: S, value: Variant<Box<RefArg>>) -> Metadata
        where
            S: Into<String>,
        {
            let mut builder = metadata_builder();
            builder.add_rest(key.into(), value);
            builder
                .finish()
                .expect("Failed to build Metadata for example")
        }

        #[test]
        fn it_supports_string_values() {
            let data = String::from("The string value");
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), "The string value".into());

            assert_eq!(metadata.rest_hash(), expected_hash);
        }

        #[test]
        fn it_supports_i64_values() {
            let data = 42i64;
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), Value::I64(42));

            assert_eq!(metadata.rest_hash(), expected_hash);
        }

        #[test]
        fn it_supports_i32() {
            let data = 42i32;
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), Value::I32(42));

            assert_eq!(metadata.rest_hash(), expected_hash);
        }

        #[test]
        fn it_supports_f64_values() {
            let data = 42.0f64;
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), Value::F64(42.0));

            assert_eq!(metadata.rest_hash(), expected_hash);
        }

        #[test]
        fn it_supports_bool_values() {
            let data = true;
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), Value::Bool(true));

            assert_eq!(metadata.rest_hash(), expected_hash);
        }

        #[test]
        fn it_stores_unknown_types() {
            let data = dbus::Path::default();
            let metadata = metadata_with_rest("foo", Variant(Box::new(data)));

            let mut expected_hash: HashMap<String, Value> = HashMap::new();
            expected_hash.insert("foo".into(), Value::Unsupported);

            assert_eq!(metadata.rest_hash(), expected_hash);
        }
    }
}
