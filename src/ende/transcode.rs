use crate::*;
use std::io::Write;

pub fn convert_json_to_msgpack(json: &[u8]) -> Result<Vec<u8>> {
    let mut ret = vec![];

    let mut deserializer = serde_json::Deserializer::from_slice(json);
    let mut serializer = rmp::Serializer::new(&mut ret);

    serde_transcode::transcode(&mut deserializer, &mut serializer)
        .c(d!())
        .and_then(|_| serializer.into_inner().flush().c(d!()))
        .map(|_| ret)
}

pub fn convert_msgpack_to_json(msgpack: &[u8]) -> Result<Vec<u8>> {
    let mut ret = vec![];

    let mut deserializer = rmp::Deserializer::new(msgpack);
    let mut serializer = serde_json::Serializer::new(&mut ret);

    serde_transcode::transcode(&mut deserializer, &mut serializer)
        .c(d!())
        .map(|_| ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ende::msgpack;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Eq, PartialEq, Deserialize, Serialize)]
    struct Foo {
        a: u32,
        b: String,
        c: (),
        d: Option<Vec<u8>>,
        e: bool,
    }

    #[test]
    fn t_convert_json_to_msgpack() {
        let foo = pnk!(do_convert_json_to_msgpack());
        assert_eq!(foo, Foo::default());
    }

    fn do_convert_json_to_msgpack() -> Result<Foo> {
        let json_str = serde_json::to_string(&Foo::default()).c(d!())?;
        let msgpack_bytes =
            super::convert_json_to_msgpack(json_str.as_bytes()).c(d!())?;
        msgpack::decode(&msgpack_bytes).c(d!())
    }

    #[test]
    fn t_convert_msgpack_to_json() {
        let foo = pnk!(do_convert_msgpack_to_json());
        assert_eq!(foo, Foo::default());
    }

    fn do_convert_msgpack_to_json() -> Result<Foo> {
        let msgpack_bytes = msgpack::encode(&Foo::default()).c(d!())?;
        let json_bytes =
            super::convert_msgpack_to_json(&msgpack_bytes).c(d!())?;
        serde_json::from_slice(&json_bytes).c(d!())
    }

    #[test]
    fn t_roundtrip_json_msgpack_json() {
        let original = Foo {
            a: 42,
            b: "hello".to_owned(),
            c: (),
            d: Some(vec![1, 2, 3]),
            e: true,
        };
        let json_bytes = serde_json::to_vec(&original).unwrap();
        let msgpack_bytes =
            super::convert_json_to_msgpack(&json_bytes).unwrap();
        let json_back =
            super::convert_msgpack_to_json(&msgpack_bytes).unwrap();
        let recovered: Foo = serde_json::from_slice(&json_back).unwrap();
        assert_eq!(original, recovered);
    }
}
