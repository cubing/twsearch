/// Defines a "newtype" for a whole number. This is useful to get an unsized
/// integer type that cannot be treated as another type without explicit
/// conversion.
///
/// This macro is public for development experimentation. Do not use outside `twsearch`.
#[macro_export]
// TODO: use inside the project without exporting.
macro_rules! whole_number_newtype {
    ($e: ident, $u_type: ident) => {
        #[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Default, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub struct $e(pub $u_type);

        // TODO: Make this a derived trait?
        impl From<$e> for $u_type {
            fn from(v: $e) -> $u_type {
                v.0
            }
        }

        // TODO: Make this a derived trait?
        impl From<$u_type> for $e {
            fn from(v: $u_type) -> Self {
                Self(v)
            }
        }

        // TODO: generalize these trait implementations?
        impl std::ops::Deref for $e {
            type Target = $u_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::Add for $e {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $e {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::str::FromStr for $e {
            type Err = <$u_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$u_type as std::str::FromStr>::from_str(s)?))
            }
        }

        impl std::ops::AddAssign for $e {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0 + rhs.0
            }
        }
    };
}

/// Defines a "newtype" for a whole number. This is useful to get an unsized
/// integer type that cannot be treated as another type without explicit
/// conversion.
///
/// This macro is public for development experimentation. Do not use outside `twsearch`.
#[macro_export]
// TODO: use inside the project without exporting.
macro_rules! whole_number_newtype_generic {
    // TODO: re-combine implementation with `whole_number_newtype`
    ($e: ident, $u_type: ident, $constraint: ident) => {
        #[allow(clippy::derived_hash_with_manual_eq)] // TODO: remove this if we can get traits derived again.
        #[derive(Clone, Debug, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub struct $e<T: $constraint>(pub $u_type, std::marker::PhantomData<T>);

        // TODO: Make this a derived trait?
        impl<T: $constraint> From<$e<T>> for $u_type {
            fn from(v: $e<T>) -> $u_type {
                v.0
            }
        }

        // TODO: Make this a derived trait?
        impl<T: $constraint> From<$u_type> for $e<T> {
            fn from(v: $u_type) -> Self {
                Self(v, PhantomData)
            }
        }

        // TODO: generalize these trait implementations?
        impl<T: $constraint> std::ops::Deref for $e<T> {
            type Target = $u_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: $constraint> std::ops::Add for $e<T> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0, PhantomData)
            }
        }

        impl<T: $constraint> std::ops::Sub for $e<T> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0, PhantomData)
            }
        }

        impl<T: $constraint> std::str::FromStr for $e<T> {
            type Err = <$u_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$u_type as std::str::FromStr>::from_str(s)?, PhantomData))
            }
        }

        impl<T: $constraint> std::ops::AddAssign for $e<T> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0 + rhs.0
            }
        }

        // TODO: is this needed?
        impl<T: $constraint> std::cmp::PartialEq for $e<T> {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        // TODO: is this needed?
        impl<T: $constraint> std::cmp::Eq for $e<T> {}

        // TODO: is this needed?
        impl<T: $constraint> std::default::Default for $e<T> {
            fn default() -> Self {
                Self(0, PhantomData)
            }
        }

        // TODO: is this needed?
        impl<T: $constraint> std::marker::Copy for $e<T> {}
    };
}
