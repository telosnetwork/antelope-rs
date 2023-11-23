use antelope_rs::chain::name::{Name, NameType};
use antelope_rs::serializer::encoder::{EncodeArgs, EncodeArgsSerializable};
use antelope_rs::serializer::Serializer;
use antelope_rs::util;

#[test]
fn name() {
    let data = "000000005c73285d";
    let object = Name::from(NameType::String("foobar".to_string()));
    let json = "foobar";

    // TODO: maybe a factory-like function or something to make creating EncodeArgs cleaner
    let arg = Box::new(object);
    let encoded = Serializer::encode(EncodeArgs::EncodeArgsSerializable(EncodeArgsSerializable { object: arg }));

    assert_eq!(encoded, util::hex_to_bytes(data));

    /*
        assert.equal(Serializer.encode({object}).hexString, data)
        assert.deepEqual(Serializer.decode({data, type: Name}), object)
        assert.deepEqual(Serializer.decode({json, type: 'name'}), object)
        assert.deepEqual(Name.from(UInt64.from('6712742083569909760')), object)
        assert.equal(JSON.stringify(object), json)
        assert.equal(object.value.toString(), '6712742083569909760')
        assert.equal(JSON.stringify(Name.from(UInt64.from(0))), '""')
     */
}