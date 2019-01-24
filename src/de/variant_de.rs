use crate::safearray::get_string_array;
use failure::{bail, Error};
use log::debug;
use serde::{de, forward_to_deserialize_any, Deserialize};
use std::fmt;
use crate::variant::Variant;

struct SeqAccess {
    data: Vec<Variant>,
    i: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
    {
        if self.i >= self.data.len() {
            return Ok(None);
        }

        let res: Variant = self.data.swap_remove(self.i);

        self.i += 1;

        seed.deserialize(res).map(Some)
    }
}

impl<'de> serde::Deserializer<'de> for Variant {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
    {
        match self {
            Variant::Null => visitor.visit_none(),
            Variant::Empty => visitor.visit_none(),
            Variant::String(s) => visitor.visit_string(s),
            Variant::I2(n) => visitor.visit_i16(n),
            Variant::I4(n) => visitor.visit_i32(n),
            Variant::I8(n) => visitor.visit_i64(n),
            Variant::Bool(b) => visitor.visit_bool(b),
            Variant::UI1(n) => visitor.visit_u8(n),
            Variant::UI8(n) => visitor.visit_u64(n),
            Variant::Array(v) => visitor.visit_seq(SeqAccess { data: v, i: 0 }),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> Deserialize<'de> for Variant {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Variant, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        struct VariantVisitor;

        impl<'de> de::Visitor<'de> for VariantVisitor {
            type Value = Variant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid variant value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Variant::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Variant::I8(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Variant::UI8(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                unimplemented!();
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Variant::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Variant::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Variant::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Variant::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: de::MapAccess<'de>,
            {
                unimplemented!()
            }
        }

        deserializer.deserialize_any(VariantVisitor)
    }
}
