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
}
