use crate::Serializer;
use crate::impossible::Impossible;
use serde::ser::{self, Serialize};

pub(crate) struct MapKeySerializer<'a> {
    pub(crate) ser: &'a mut Serializer,
}

impl<'a> ser::Serializer for MapKeySerializer<'a> {
    type Ok = ();
    type Error = serde::de::value::Error;

    type SerializeSeq = Impossible;
    type SerializeTuple = Impossible;
    type SerializeTupleStruct = Impossible;
    type SerializeTupleVariant = Impossible;
    type SerializeMap = Impossible;
    type SerializeStruct = Impossible;
    type SerializeStructVariant = Impossible;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.ser
            .write_string_value(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&v.to_string());
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(&String::from_utf8_lossy(v));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value("null");
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self.ser)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value("");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(name);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self.ser)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.ser.write_string_value(variant);
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Impossible, Self::Error> {
        unreachable!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Impossible, Self::Error> {
        unreachable!()
    }
}
