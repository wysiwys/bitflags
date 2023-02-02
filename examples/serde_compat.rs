//! An example of implementing `serde::Serialize` and `serde::Deserialize` equivalently to how
//! `#[derive(Serialize, Deserialize)]` would on `bitflags` `1.x` types.

#[cfg(feature = "serde")]
fn main() {
    bitflags::bitflags! {
        // Removed: `serde` traits from the `#[derive]` attribute
        // #[derive(Serialize, Deserialize)]
        #[derive(Debug, PartialEq, Eq)]
        pub struct Flags: u32 {
            const A = 1;
            const B = 2;
            const C = 4;
            const D = 8;
        }
    }

    // Added: Manual `Serialize` and `Deserialize` implementations based on a generic impl
    impl serde::Serialize for Flags {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            legacy_format::serialize(self, serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Flags {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            legacy_format::deserialize(deserializer)
        }
    }

    pub mod legacy_format {
        //! This module is a generic implementation of `Serialize` and `Deserialize` that can be used by
        //! any flags type generated by `bitflags!`.
        //!
        //! Don't be intimidated by the amount of `serde` code in here! It boils down to serializing and deserializing
        //! a struct with a single `bits` field. It may be converted into a library at some point, but is also suitable
        //! to copy into your own project if you need it.

        use core::{any::type_name, fmt};
        use serde::{
            de::{Error, MapAccess, Visitor},
            ser::SerializeStruct,
            Deserialize, Deserializer, Serialize, Serializer,
        };

        use bitflags::BitFlags;

        /// Serialize a flags type equivalently to how `#[derive(Serialize)]` on a flags type
        /// from `bitflags` `1.x` would.
        pub fn serialize<T: BitFlags, S: Serializer>(
            flags: &T,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            <T as BitFlags>::Bits: Serialize,
        {
            let mut serialize_struct = serializer.serialize_struct(type_name::<T>(), 1)?;
            serialize_struct.serialize_field("bits", &flags.bits())?;
            serialize_struct.end()
        }

        /// Deserialize a flags type equivalently to how `#[derive(Deserialize)]` on a flags type
        /// from `bitflags` `1.x` would.
        pub fn deserialize<'de, T: BitFlags, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<T, D::Error>
        where
            <T as BitFlags>::Bits: Deserialize<'de>,
        {
            struct BitsVisitor<T>(core::marker::PhantomData<T>);

            impl<'de, T: Deserialize<'de>> Visitor<'de> for BitsVisitor<T> {
                type Value = T;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a primitive bitflags value wrapped in a struct")
                }

                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut bits = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            "bits" => {
                                if bits.is_some() {
                                    return Err(Error::duplicate_field("bits"));
                                }

                                bits = Some(map.next_value()?);
                            }
                            v => return Err(Error::unknown_field(v, &["bits"])),
                        }
                    }

                    bits.ok_or_else(|| Error::missing_field("bits"))
                }
            }

            let bits = deserializer.deserialize_struct(
                type_name::<T>(),
                &["bits"],
                BitsVisitor(Default::default()),
            )?;

            Ok(T::from_bits_retain(bits))
        }
    }

    let flags = Flags::A | Flags::B;

    let serialized = serde_json::to_string(&flags).unwrap();

    println!("{:?} -> {}", flags, serialized);

    assert_eq!(serialized, r#"{"bits":3}"#);

    let deserialized: Flags = serde_json::from_str(&serialized).unwrap();

    println!("{} -> {:?}", serialized, flags);

    assert_eq!(deserialized, flags);
}

#[cfg(not(feature = "serde"))]
fn main() {}
