#[macro_export]
macro_rules! make_enum {
    ($name: ident <$word: ty> { $($op: ident = $code: expr),+ $(,)? }) => {
        #[derive(Debug)]
        #[repr($word)]
        pub enum $name {
            $($op = $code),+
        }

        impl $name {
            pub fn value(self) -> $word {
                self.into()
            }
        }

        impl Into<$word> for $name {
            fn into(self) -> $word {
                self as $word
            }
        }

        impl TryFrom<$word> for $name {
            type Error = ProtocolError;
            fn try_from(value: $word) -> Result<Self, ProtocolError> {
                match value {
                    $($code => Ok(Self::$op)),+,
                    _ => Err(ProtocolError::InvalidProtocol(value))
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}
