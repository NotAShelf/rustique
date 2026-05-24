use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            #[must_use]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<Path> for $name {
            fn as_ref(&self) -> &Path {
                Path::new(&self.0)
            }
        }

        impl AsRef<OsStr> for $name {
            fn as_ref(&self) -> &OsStr {
                OsStr::new(&self.0)
            }
        }

        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl Borrow<String> for $name {
            fn borrow(&self) -> &String {
                &self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                &self.0 == other
            }
        }

        impl std::ops::Add<&str> for $name {
            type Output = String;

            fn add(self, rhs: &str) -> Self::Output {
                self.0 + rhs
            }
        }
    };
}

string_newtype!(ModID);
string_newtype!(ModName);
string_newtype!(ModVersion);
string_newtype!(ModFileName);
string_newtype!(DownloadURL);
string_newtype!(FileName);
string_newtype!(UrlString);

pub type Tags = Vec<String>;
/// Used with the parse_{pinned,latest}_version functions
pub type PinnedVersionInfo = (ModVersion, DownloadURL, Tags, String);
