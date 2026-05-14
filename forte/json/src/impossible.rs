use serde::ser::{self, Serialize};

pub(crate) enum Impossible {}

impl ser::SerializeSeq for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_element<T: Serialize + ?Sized>(
        &mut self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeTuple for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_element<T: Serialize + ?Sized>(
        &mut self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeTupleStruct for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeTupleVariant for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeMap for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, _key: &T) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn serialize_value<T: Serialize + ?Sized>(
        &mut self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeStruct for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

impl ser::SerializeStructVariant for Impossible {
    type Ok = ();
    type Error = serde::de::value::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        match *self {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}
