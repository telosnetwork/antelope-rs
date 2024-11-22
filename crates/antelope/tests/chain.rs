use antelope::chain::authority::{Authority, KeyWeight, WaitWeight};
use antelope::chain::key_type::KeyType;
use antelope::chain::private_key::PrivateKey;
use antelope::chain::public_key::PublicKey;
use antelope::chain::signature::Signature;
use antelope::{
    chain::{
        action::{Action, PermissionLevel},
        asset::{Asset, Symbol},
        block_id::BlockId,
        checksum::{Checksum160, Checksum256, Checksum512},
        name::Name,
        transaction::{Transaction, TransactionHeader},
        Decoder, Encoder, Packer,
    },
    name,
    util::{bytes_to_hex, hex_to_bytes},
};
use antelope_client_macros::StructPacker;

#[test]
fn asset() {
    // TODO: Should asset support negative values?
    //assert_eq!(Asset::from_string("-1.2345 NEGS").to_string(), "-1.2345 NEGS");
    //assert.equal(Asset.from('-1.2345 NEGS').toString(), '-1.2345 NEGS')
    //assert.equal(Asset.from('-0.2345 NEGS').toString(), '-0.2345 NEGS')
    //assert.equal(Asset.from('-99999999999 DUCKS').toString(), '-99999999999
    // DUCKS') assert.equal(Asset.from('-0.0000000000001 DUCKS').toString(),
    // '-0.0000000000001 DUCKS')
    assert_eq!(
        Asset::from_string("0.0000000000000 DUCKS").to_string(),
        "0.0000000000000 DUCKS"
    );
    assert_eq!(
        Asset::from_string("99999999999 DUCKS").to_string(),
        "99999999999 DUCKS"
    );

    let asset = Asset::from_string("1.000000000 FOO");
    assert_eq!(asset.amount(), 1000000000);
    let new_asset = asset + asset;
    assert_eq!(new_asset.amount(), 2000000000);
    /* TODO: Support negative?
    asset.value = -100
    assert.equal(asset.toString(), '-100.000000000 FOO')
    assert.equal(asset.units.toString(), '-100000000000')
    */

    let symbol = Symbol::new("K", 10);
    assert_eq!(symbol.code().to_string(), "K");
    assert_eq!(symbol.precision(), 10);
    assert_eq!(symbol.to_string(), "10,K");

    let symbol_bytes = Encoder::pack(&symbol);
    let mut symbol_decoder = Decoder::new(symbol_bytes.as_slice());
    let symbol_unpacked = &mut Symbol::default();
    symbol_decoder.unpack(symbol_unpacked);
    assert_eq!(symbol.to_string(), symbol_unpacked.to_string());
    /*
       // test null asset
       asset = Asset.from('0 ')
       assert.equal(Number(asset.value), 0)
       assert.equal(String(asset), '0 ')

       asset = Asset.from(10, '4,POX')
       assert.equal(asset.value, 10)
       assert.equal(Number(asset.units), 100000)

       asset = Asset.fromUnits(1, '10,KEK')
       assert.equal(asset.value, 0.0000000001)
       asset.value += 0.0000000001
       assert.equal(Number(asset.units), 2)

       asset = Asset.from(3.004, '4,RAR')
       asset.value += 1
       assert.equal(asset.toString(), '4.0040 RAR')
       assert.equal(asset.value, 4.004)

       asset = Asset.from(3.004, '8,RAR')
       asset.value += 1
       assert.equal(asset.units.toNumber(), 400400000)
       assert.equal(asset.toString(), '4.00400000 RAR')
       assert.equal(asset.value, 4.004)

       assert.throws(() => {
           symbol.convertUnits(Int64.from('9223372036854775807'))
       })
       assert.throws(() => {
           Asset.from('')
       })
       assert.throws(() => {
           Asset.from('1POP')
       })
       assert.throws(() => {
           Asset.from('1.0000000000000000000000 BIGS')
       })
       assert.throws(() => {
           Asset.from('1.2 horse')
       })
       assert.throws(() => {
           Asset.Symbol.from('12')
       })
    */
}

#[test]
fn block_id() {
    let string = "048865fb643bca3b644647177f0cf363f7956794d0a7ec3bc6d29d93d9637308";

    let block_id_bytes = hex::decode(string).unwrap();
    let block_id = BlockId::from_bytes(&block_id_bytes).unwrap();

    assert_eq!(block_id.block_num().to_string(), "76047867");
    assert_eq!(block_id.block_num(), 76047867);

    //assert!(block_id.block_num().equals(UInt32::from(76047867))); UInt32 not
    // implemented yet

    //decode not implemented yet
    // let block_id2 = BlockId::from(BlockIdType::BlockChecksumAndNumber {
    //     checksum: Checksum256::from(hex::decode(
    //         "61375f2d5fbe6bbad86e424962a190e8309394b7bff4bf3e16b0a2a71e5a617c",
    //     )
    //     .unwrap()),
    //     block_num: UInt32::from(7),
    // });

    // assert_eq!(block_id2.to_string(),
    // "000000075fbe6bbad86e424962a190e8309394b7bff4bf3e16b0a2a71e5a617c");
    // assert!(block_id2.block_num().equals(7));
}

/*  test('block id', function () {
    const string = '048865fb643bca3b644647177f0cf363f7956794d0a7ec3bc6d29d93d9637308'
    const blockId = BlockId.from(string)
    assert.equal(String(blockId), string)
    assert.equal(Number(blockId.blockNum), 76047867)
    assert.equal(blockId.blockNum.equals(76047867), true)
    assert.equal(blockId.blockNum.equals(UInt32.from(76047867)), true)
    const blockId2 = BlockId.fromBlockChecksum(
        '61375f2d5fbe6bbad86e424962a190e8309394b7bff4bf3e16b0a2a71e5a617c',
        7
    )
    assert.equal(
        String(blockId2),
        '000000075fbe6bbad86e424962a190e8309394b7bff4bf3e16b0a2a71e5a617c'
    )
    assert.equal(blockId2.blockNum.equals(7), true)
})*/

/*
#[test]
fn blob() {
    let expected = Blob::from(BlobType::Bytes(vec![0xbe, 0xef, 0xfa, 0xce])).unwrap();

    //Correct
    let blob = Blob::from(BlobType::String("vu/6zg==".to_string())).unwrap();
    assert_eq!(blob.array, expected.array);

    // Wrong padding, ensure it still works
    let blob2 = Blob::from(BlobType::String("vu/6zg=".to_string())).unwrap();
    assert_eq!(blob2.array, expected.array);

    let blob3 = Blob::from(BlobType::String("vu/6zg".to_string())).unwrap();
    assert_eq!(blob3.array, expected.array);

    let blob4 = Blob::from(BlobType::String("vu/6zg===".to_string())).unwrap();
    assert_eq!(blob4.array, expected.array);
}
*/

/*    test('blob', function () {
    const expected = Bytes.from([0xbe, 0xef, 0xfa, 0xce])

    // Correct
    const string = 'vu/6zg=='
    const blob = Blob.from(string)
    assert.isTrue(Bytes.from(blob.array).equals(expected))

    // Wrong padding, ensure it still works
    const string2 = 'vu/6zg='
    const blob2 = Blob.from(string2)
    assert.isTrue(Bytes.from(blob2.array).equals(expected))

    const string3 = 'vu/6zg'
    const blob3 = Blob.from(string3)
    assert.isTrue(Bytes.from(blob3.array).equals(expected))

    const string4 = 'vu/6zg==='
    const blob4 = Blob.from(string4)
    assert.isTrue(Bytes.from(blob4.array).equals(expected))
})
*/

#[test]
fn bytes() {
    let hello_world_bytes = "hello world".as_bytes();
    let hello_bytes = "hello".as_bytes();
    assert_eq!(bytes_to_hex(&hello_bytes.to_vec()), "68656c6c6f");

    assert_eq!(
        Checksum256::hash(hello_world_bytes.to_vec()).to_string(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
    assert_eq!(
        Checksum512::hash(hello_world_bytes.to_vec()).to_string(),
        "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
    );
    assert_eq!(
        Checksum160::hash(hello_world_bytes.to_vec()).to_string(),
        "98c615784ccb5fe5936fbc0cbe9dfdb408d92f0f"
    );
    /*
       // TODO: add zeropadded support
       assert.equal(Bytes.from('beef').zeropadded(4).toString('hex'), '0000beef')
       assert.equal(Bytes.from('beef').zeropadded(2).toString('hex'), 'beef')
       assert.equal(Bytes.from('beef').zeropadded(1).toString('hex'), 'beef')
       assert.equal(Bytes.from('beef').zeropadded(1, true).toString('hex'), 'be')
       assert.equal(Bytes.from('beef').zeropadded(2, true).toString('hex'), 'beef')
       assert.equal(Bytes.from('beef').zeropadded(3, true).toString('hex'), '00beef')

    */
}

/*

test('time', function () {
    const now = new Date()
    assert.equal(TimePoint.from(now).toMilliseconds(), now.getTime())
    assert.equal(
        TimePointSec.from(TimePointSec.from(now)).toMilliseconds() / 1000,
        Math.round(now.getTime() / 1000)
    )
    assert.throws(() => {
        TimePoint.from('blah')
    })
    assert.equal(BlockTimestamp.from('2021-08-25T02:37:24.500'), '2021-08-25T02:37:24.500')
    assert.equal(
        Math.round(BlockTimestamp.from(now).toMilliseconds() / 500),
        Math.round(now.getTime() / 500)
    )
})
*/

#[test]
fn transaction() {
    #[derive(Clone, Eq, PartialEq, Default, StructPacker)]
    struct Transfer {
        from: Name,
        to: Name,
        quantity: Asset,
        memo: String,
    }

    let transfer_data = Transfer {
        from: name!("foo"),
        to: name!("bar"),
        quantity: Asset::from_string("1.0000 EOS"),
        memo: String::from("hello"),
    };

    let transfer_data_packed = Encoder::pack(&transfer_data);
    assert_eq!(
        bytes_to_hex(&transfer_data_packed),
        "000000000000285d000000000000ae39102700000000000004454f53000000000568656c6c6f"
    );

    let action = Action::new_ex(
        name!("eosio.token"),
        name!("transfer"),
        vec![],
        transfer_data,
    );

    let action_packed = Encoder::pack(&action);
    assert_eq!(bytes_to_hex(&action_packed), "00a6823403ea3055000000572d3ccdcd0026000000000000285d000000000000ae39102700000000000004454f53000000000568656c6c6f");

    let transaction = Transaction {
        header: TransactionHeader::default(),
        context_free_actions: vec![],
        actions: vec![action],
        extension: vec![],
    };
    assert_eq!(
        bytes_to_hex(&transaction.id()),
        "97b4d267ce0e0bd6c78c52f85a27031bd16def0920703ca3b72c28c2c5a1a79b"
    );

    let transfer_decoded = &mut Transfer::default();
    let mut decoder = Decoder::new(&transaction.actions[0].data);
    decoder.unpack(transfer_decoded);
    assert_eq!(transfer_decoded.from, name!("foo"));
    /*

    const signed = SignedTransaction.from({
        ...transaction,
        signatures: [
            'SIG_K1_KdNTcLLSyzUFC4AdMxEDn58X8ZN368euanvet4jucUdSPXvLkgsG32tpcqVvnDR9Xv1f7HsTm6kocjeZzFGvUSc2yCbdEA',
        ],
    })
    assert.equal(String(signed.id), String(transaction.id))

     */
}

/*
    test('any transaction', function () {
        const tx: AnyTransaction = {
            delay_sec: 0,
            expiration: '2020-07-01T17:32:13',
            max_cpu_usage_ms: 0,
            max_net_usage_words: 0,
            ref_block_num: 55253,
            ref_block_prefix: 3306698594,
            actions: [
                {
                    account: 'eosio.token',
                    name: 'transfer',
                    authorization: [{actor: 'foo', permission: 'active'}],
                    data: {
                        from: 'donkeyhunter',
                        memo: 'Anchor is the best! Thank you <3',
                        quantity: '0.0001 EOS',
                        to: 'teamgreymass',
                    },
                },
            ],
        }
        const abi: ABIDef = {
            structs: [
                {
                    base: '',
                    name: 'transfer',
                    fields: [
                        {name: 'from', type: 'name'},
                        {name: 'to', type: 'name'},
                        {name: 'quantity', type: 'asset'},
                        {name: 'memo', type: 'string'},
                    ],
                },
            ],
            actions: [{name: 'transfer', type: 'transfer', ricardian_contract: ''}],
        }
        const r1 = Transaction.from(tx, abi)
        const r2 = Transaction.from(tx, [{abi, contract: 'eosio.token'}])
        assert.equal(r1.equals(r2), true)
        assert.deepEqual(
            JSON.parse(JSON.stringify(r1.actions[0].decodeData(abi))),
            tx.actions![0].data
        )
        assert.throws(() => {
            Transaction.from(tx)
        })
        assert.throws(() => {
            Transaction.from(tx, [{abi, contract: 'ethereum.token'}])
        })
    })
    test('random', function () {
        assert.doesNotThrow(() => {
            UInt128.random()
            Int32.random()
        })
        assert.equal(UInt128.random().byteArray.length, 16)
    })

    test('equality helpers', function () {
        this.slow(500)

        const name = Name.from('foo')
        assert.equal(name.equals('foo'), true)
        assert.equal(name.equals(UInt64.from('6712615244595724288')), true)
        assert.equal(name.equals(UInt64.from('12345')), false)
        assert.equal(name.equals('bar'), false)

        const num = UInt64.from('123456789')
        assert.equal(num.equals(123456789), true)
        assert.equal(num.equals('123456789'), true)
        assert.equal(num.equals('123456700'), false)
        assert.equal(num.equals(1), false)
        assert.equal(num.equals(UInt32.from(123456789)), true)
        assert.equal(num.equals(UInt32.from(123456789), true), false)
        assert.equal(num.equals(UInt128.from(123456789), true), false)
        assert.equal(num.equals(UInt128.from(123456789), false), true)
        assert.equal(num.equals(Int64.from(123456789), true), false)
        assert.equal(num.equals(-1), false)

        const checksum = Checksum160.hash(Bytes.from('hello', 'utf8'))
        assert.equal(checksum.equals('108f07b8382412612c048d07d13f814118445acd'), true)
        assert.equal(checksum.equals('108f07b8382412612c048d07d13f814118445abe'), false)

        const pubKey = PublicKey.from('EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin')
        assert.equal(
            pubKey.equals('PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs'),
            true
        )

        const sig = Signature.from(
            'SIG_K1_JyMXe1HU42qN2aM7GPUf5XrAcAjWPbRoojzfsKq9Rgto3dGsRcCZ4UaPsAcFPS2faGQMpRoSTRX8WQQUDEA5TfWHj8sr6q'
        )
        assert.equal(
            sig.equals(
                'SIG_K1_JyMXe1HU42qN2aM7GPUf5XrAcAjWPbRoojzfsKq9Rgto3dGsRcCZ4UaPsAcFPS2faGQMpRoSTRX8WQQUDEA5TfWHj8sr6q'
            ),
            true
        )
        assert.equal(
            sig.equals(
                'SIG_R1_K5VEcCFUxF2jptQJUjVhV99PNiBXur6kdz6xuHtqvjqoTnzGqcCkEpD6cuA4q9DPdEHysdXjfksLB5xfkERxBuWxb9QJ8y'
            ),
            false
        )
*/

// fn print_values(perm: &PermissionLevel, other_perm: &PermissionLevel) {
//     info!("------Testing to_string()------");
//     info!("Permission 1: {}", perm.to_string());
//     info!("Permission 2: {}", other_perm.to_string());

//     info!("--------Testing json()--------");
//     info!("Permission 1: {:?}", perm.to_json());
//     info!("Permission 2: {:?}", other_perm.to_json());
// }

#[test]
fn permission_level() {
    // Create PermissionLevel from 'foo@bar'
    let perm = PermissionLevel::new(Name::new_from_str("foo"), Name::new_from_str("bar"));

    // Test equals with itself
    assert_eq!(perm, perm.clone());

    // Test equals with equivalent ActorPermission
    let other_perm = PermissionLevel::new(Name::new_from_str("foo"), Name::new_from_str("bar"));
    assert_eq!(perm, other_perm);

    // Test equals with different PermissionLevel
    let different_perm = PermissionLevel::new(Name::new_from_str("bar"), Name::new_from_str("moo"));
    assert_ne!(perm, different_perm);
}

/*
        const perm = PermissionLevel.from('foo@bar')
        assert.equal(perm.equals(perm), true)
        assert.equal(perm.equals({actor: 'foo', permission: 'bar'}), true)
        assert.equal(perm.equals('bar@moo'), false)

        @Struct.type('my_struct')
        class MyStruct extends Struct {
            @Struct.field('string') hello!: string
        }
        const struct = MyStruct.from({hello: 'world'})
        assert.equal(struct.equals(struct), true)
        assert.equal(struct.equals({hello: 'world'}), true)
        assert.equal(struct.equals({hello: 'bollywod'}), false)

        @Variant.type('my_variant', ['string', 'int32'])
        class MyVariant extends Variant {
            value!: string | Int32
        }
        const variant = MyVariant.from('hello')
        assert.equal(variant.equals(variant), true)
        assert.equal(variant.equals('hello'), true)
        assert.equal(variant.equals('boo'), false)
        assert.equal(variant.equals(Int32.from(1)), false)
        assert.equal(variant.equals(MyVariant.from('haj')), false)

        @Struct.type('my_struct')
        class MyStructWithVariant extends Struct {
            @Struct.field(MyVariant) field!: MyVariant
        }
        const action = Action.from({
            account: 'foo',
            name: 'bar',
            authorization: [perm],
            data: MyStructWithVariant.from({
                field: variant,
            }),
        })
        assert.equal(action.equals(action), true)
        assert.equal(
            action.equals({
                account: 'foo',
                name: 'bar',
                authorization: [perm],
                data: {
                    field: 'hello',
                },
            }),
            true
        )
        assert.equal(
            action.equals({
                account: 'foo',
                name: 'bar',
                authorization: [perm],
                data: {
                    field: variant,
                },
            }),
            true
        )
        assert.equal(
            action.equals({
                account: 'foo',
                name: 'bar',
                authorization: [],
                data: {
                    field: variant,
                },
            }),
            false
        )
        assert.equal(
            action.equals({
                account: 'foo',
                name: 'bar',
                authorization: [{actor: 'maa', permission: 'jong'}],
                data: {
                    field: variant,
                },
            }),
            false
        )

        const time = TimePointSec.from(1)
        assert.equal(time.equals(time), true)
        assert.equal(time.equals('1970-01-01T00:00:01'), true)
        assert.equal(time.equals('2020-02-20T02:20:20'), false)
        assert.equal(time.equals(1), true)
        assert.equal(time.equals(2), false)
        assert.equal(time.equals(TimePoint.from(1 * 1000000)), true)
    })

*/

#[test]
fn transaction_signing_data_and_digest() {
    let trx = Transaction {
        header: TransactionHeader {
            expiration: Default::default(),
            ref_block_num: 0,
            ref_block_prefix: 0,
            max_net_usage_words: Default::default(),
            max_cpu_usage_ms: 0,
            delay_sec: Default::default(),
        },
        context_free_actions: vec![],
        actions: vec![Action {
            account: name!("eosio.token"),
            name: name!("transfer"),
            authorization: vec![PermissionLevel::new(name!("corecorecore"), name!("active"))],
            data: hex_to_bytes(
                "a02e45ea52a42e4580b1915e5d268dcaba0100000000000004454f530000000019656f73696f2d636f7265206973207468652062657374203c33",
            ),
        }],
        extension: vec![],
    };
    let chain_id = Checksum256::from_bytes(
        hex_to_bytes("2a02a0053e5a8cf73a56ba0fda11e4d92e0238a4a2aa74fccf46d5a910746840").as_slice(),
    )
    .unwrap();
    let data = trx.signing_data(chain_id.data.as_ref());
    let expected_data_hex= "2a02a0053e5a8cf73a56ba0fda11e4d92e0238a4a2aa74fccf46d5a91074684000000000000000000000000000000100a6823403ea3055000000572d3ccdcd01a02e45ea52a42e4500000000a8ed32323aa02e45ea52a42e4580b1915e5d268dcaba0100000000000004454f530000000019656f73696f2d636f7265206973207468652062657374203c33000000000000000000000000000000000000000000000000000000000000000000";
    assert_eq!(bytes_to_hex(&data), expected_data_hex);

    let digest = trx.signing_digest(chain_id.data.as_ref());
    let expected_digest_hex = "59fa6b615e3ce1b539ae27bc2398448c1374d2d3c97fe2bbba2c37c118631848";
    assert_eq!(bytes_to_hex(&digest), expected_digest_hex);
}

#[test]
fn transaction_signature_verification() {
    // ID of the devnet chain (`telos-native-docker``)
    let chain_id = [75, 228, 192, 251, 167, 108, 89, 177, 114, 187, 74, 20, 103, 30, 226, 3, 44, 14, 209, 54, 110, 249, 22, 159, 202, 188, 155, 52, 52, 74, 36, 18];

    // Encoded transaction generated by the web client of the `telos-zk-demo`
    let encoded_transaction = [218, 181, 61, 103, 113, 47, 244, 109, 91, 94, 0, 0, 0, 0, 1, 96, 213, 101, 204, 0, 234, 48, 85, 0, 46, 99, 38, 99, 165, 173, 186, 1, 0, 0, 0, 0, 0, 133, 92, 52, 0, 0, 0, 0, 168, 237, 50, 50, 19, 2, 171, 205, 7, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0];
    println!("Encoded transaction: {:?}", bytes_to_hex(&encoded_transaction.to_vec()));
    println!();

    // Encoded signature generated by the Anchor wallet from the `encoded_transaction` above
    let encoded_signature = [31, 1, 107, 241, 2, 44, 130, 235, 22, 119, 107, 84, 67, 135, 122, 124, 130, 179, 237, 143, 210, 238, 42, 1, 176, 28, 224, 105, 182, 161, 117, 254, 41, 36, 241, 25, 104, 38, 143, 26, 250, 248, 161, 23, 2, 81, 105, 133, 72, 229, 69, 98, 114, 165, 90, 5, 139, 199, 122, 171, 74, 112, 29, 24, 9];

    // Decoding the transaction
    let mut transaction = Transaction::default();
    transaction.unpack(encoded_transaction.as_slice());

    println!("CHAIN ID: {}", bytes_to_hex(&chain_id.to_vec()));
    println!();

    // Decoding checks
    {
        let action = transaction.actions.first().unwrap();
        let sender = action.authorization.first().unwrap().actor.as_string();
        println!("Contract account: {}", action.account.as_string());
        println!("Action name: {}", action.name.as_string());
        println!("Sender: {}", sender);
        println!("Request data: {:?}", action.data);
        println!();

        assert_eq!(action.account.as_string(), "eosio.aggreq");
        assert_eq!(action.name.as_string(), "requestaggr");
        assert_eq!(sender, "alice");
        assert_eq!(action.data, vec![2, 171, 205, 7, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0]);
    }

    // Alice account checks
    {
        let private_key = PrivateKey::from_str("5KWu5C8FDdNcoCLta3hXuyDKJcxAgaaza3MLkwRJWwEz9C2dn5u", false).unwrap();
        let public_key = private_key.to_public();
        assert_eq!(public_key.to_legacy_string(None).unwrap(), "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc");

        // Create a sample signature
        let message = b"I like turtles";
        let signature = private_key.sign_message(&message.to_vec());
        let recovered_public_key = signature.recover_message(&message.to_vec());
        assert_eq!(recovered_public_key.to_legacy_string(None).unwrap(), "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc");

        // Create the signature of the transaction
        let message = transaction.signing_digest(&chain_id);
        let signature = private_key.sign_message(&message.to_vec());
        println!("Alice signature: {}", signature.to_string());
        let recovered_public_key = signature.recover_message(&message.to_vec());
        assert_eq!(recovered_public_key.to_legacy_string(None).unwrap(), "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc");
    }

    // Signature checks
    {
        // Check that the computed signing data matches the expected value
        let mut signing_data = transaction.signing_data(&chain_id);
        let expected_signing_data = [75, 228, 192, 251, 167, 108, 89, 177, 114, 187, 74, 20, 103, 30, 226, 3, 44, 14, 209, 54, 110, 249, 22, 159, 202, 188, 155, 52, 52, 74, 36, 18, 218, 181, 61, 103, 113, 47, 244, 109, 91, 94, 0, 0, 0, 0, 1, 96, 213, 101, 204, 0, 234, 48, 85, 0, 46, 99, 38, 99, 165, 173, 186, 1, 0, 0, 0, 0, 0, 133, 92, 52, 0, 0, 0, 0, 168, 237, 50, 50, 19, 2, 171, 205, 7, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        println!("Signing data: {:?}", signing_data);
        assert_eq!(signing_data, expected_signing_data);

        // Check that the computed signing digest matches the expected value
        let signing_digest = transaction.signing_digest(&chain_id);
        let expected_signing_digest = [240, 149, 129, 84, 234, 50, 126, 42, 230, 106, 37, 117, 141, 156, 68, 69, 217, 199, 21, 201, 192, 134, 23, 148, 213, 178, 204, 247, 225, 232, 113, 190];
        println!("Signing digest: {:?}", signing_digest);
        assert_eq!(signing_digest, expected_signing_digest);

        // Decode the signature
        let signature = Signature::from_bytes(encoded_signature.to_vec(), KeyType::K1);
        println!("Expected signature: {}", signature.to_string());
        assert_eq!(signature.to_string(), "SIG_K1_JuSZfHNg6b68ag1znsoJvBARmqMR34AJ6KPpZMoiEFZ38paAqQpwiqjmen7yFkEefWNVWqjD3pCJrAntXXDLkNkpxe8Uyf");

        // Try to compute the same signature
        let private_key = PrivateKey::from_str("5KWu5C8FDdNcoCLta3hXuyDKJcxAgaaza3MLkwRJWwEz9C2dn5u", true).unwrap();
        let computed_signature = private_key.sign_message(&signing_data);
        println!("Computed signature: {}", computed_signature.to_string());
        assert_eq!(computed_signature, signature);

        // Recover the public key from the signature
        let public_key = signature.recover_message(&signing_data);
        println!("Public key: {}", public_key.to_legacy_string(None).unwrap());
        assert_eq!(public_key.to_legacy_string(None).unwrap(), "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc");

        // Verify the signature
        assert!(signature.verify_message(&signing_data, &public_key));

        // Check that the verification fails if we change a byte of the signing digest
        signing_data[0] += 1;
        assert!(!signature.verify_message(&signing_data, &public_key));
    }
}

/*
test('action with no arguments', function () {
    const abi = {
        structs: [{name: 'noop', base: '', fields: []}],
        actions: [
            {
                name: 'noop',
                type: 'noop',
                ricardian_contract: '',
            },
        ],
    }
    const a1 = Action.from(
        {
            account: 'greymassnoop',
            name: 'noop',
            authorization: [{actor: 'greymassfuel', permission: 'cosign'}],
            data: '',
        },
        abi
    )
    const a2 = Action.from(
        {
            account: 'greymassnoop',
            name: 'noop',
            authorization: [{actor: 'greymassfuel', permission: 'cosign'}],
            data: {},
        },
        abi
    )
    const a3 = Action.from(
        {
            account: 'greymassnoop',
            name: 'noop',
            authorization: [{actor: 'greymassfuel', permission: 'cosign'}],
            data: [],
        },
        abi
    )
    assert.equal(a1.equals(a2), true)
    assert.equal(a1.equals(a3), true)
})

test('action retains abi (abi)', function () {
    const abi = {
        structs: [{name: 'noop', base: '', fields: []}],
        actions: [
            {
                name: 'noop',
                type: 'noop',
                ricardian_contract: '',
            },
        ],
    }
    const action = Action.from(
        {
            account: 'greymassnoop',
            name: 'noop',
            authorization: [{actor: 'greymassfuel', permission: 'cosign'}],
            data: '',
        },
        abi
    )
    assert.instanceOf(action.abi, ABI)
})

test('action can deserialize itself from abi', function () {
    const abi = {
        structs: [
            {
                name: 'transfer',
                base: '',
                fields: [
                    {
                        name: 'from',
                        type: 'name',
                    },
                    {
                        name: 'to',
                        type: 'name',
                    },
                    {
                        name: 'quantity',
                        type: 'asset',
                    },
                    {
                        name: 'memo',
                        type: 'string',
                    },
                ],
            },
        ],
        actions: [
            {
                name: 'transfer',
                type: 'transfer',
                ricardian_contract: '',
            },
        ],
    }

    const action = Action.from(
        {
            account: 'eosio.token',
            name: 'transfer',
            authorization: [{actor: 'foo', permission: 'bar'}],
            data: {
                from: 'foo',
                to: 'bar',
                quantity: '1.0000 EOS',
                memo: 'hello',
            },
        },
        abi
    )
    assert.instanceOf(action.abi, ABI)
    const decoded = action.decoded
    assert.instanceOf(decoded.account, Name)
    assert.instanceOf(decoded.name, Name)
    assert.instanceOf(decoded.authorization, Array)
    assert.instanceOf(decoded.authorization[0], PermissionLevel)
    assert.instanceOf(decoded.data.from, Name)
    assert.instanceOf(decoded.data.to, Name)
    assert.instanceOf(decoded.data.quantity, Asset)
})

test('action retains abi (struct)', function () {
    @Struct.type('transfer')
    class Transfer extends Struct {
        @Struct.field('name') from!: Name
        @Struct.field('name') to!: Name
        @Struct.field('asset') quantity!: Asset
        @Struct.field('string') memo!: string
    }

    const data = Transfer.from({
        from: 'foo',
        to: 'bar',
        quantity: '1.0000 EOS',
        memo: 'hello',
    })

    const action = Action.from({
        authorization: [],
        account: 'eosio.token',
        name: 'transfer',
        data,
    })
    assert.instanceOf(action.abi, ABI)

    const transaction = Transaction.from({
        ref_block_num: 0,
        ref_block_prefix: 0,
        expiration: 0,
        actions: [action],
    })

    assert.instanceOf(transaction.actions[0].abi, ABI)
    assert.isTrue(action.equals(transaction.actions[0]))
    assert.isTrue(transaction.actions[0].equals(action))
    assert.isTrue(
        data.equals(
            Serializer.decode({
                data: transaction.actions[0].data,
                abi: transaction.actions[0].abi,
                type: String(transaction.actions[0].name),
            })
        )
    )
})

test('action can deserialize itself from struct', function () {
    @Struct.type('transfer')
    class Transfer extends Struct {
        @Struct.field('name') from!: Name
        @Struct.field('name') to!: Name
        @Struct.field('asset') quantity!: Asset
        @Struct.field('string') memo!: string
    }
    const data = Transfer.from({
        from: 'foo',
        to: 'bar',
        quantity: '1.0000 EOS',
        memo: 'hello',
    })

    const action = Action.from({
        authorization: [
            {
                actor: 'foo',
                permission: 'bar',
            },
        ],
        account: 'eosio.token',
        name: 'transfer',
        data,
    })
    assert.instanceOf(action.abi, ABI)
    const decoded = action.decoded
    assert.instanceOf(decoded.account, Name)
    assert.instanceOf(decoded.name, Name)
    assert.instanceOf(decoded.authorization, Array)
    assert.instanceOf(decoded.authorization[0], PermissionLevel)
    assert.instanceOf(decoded.data.from, Name)
    assert.instanceOf(decoded.data.to, Name)
    assert.instanceOf(decoded.data.quantity, Asset)
})
*/

#[test]
fn authority() {
    let data = "15000000020002caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc140001039e338bb411813f6b10a9d5dac9cf1afeedc698ff5726f81b521f6c328459b936020000010a0000000100";
    let auth = Authority {
        threshold: 21,
        keys: vec![
            KeyWeight {
                key: PublicKey::new_from_str(
                    "EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin",
                )
                .unwrap(),
                weight: 20,
            },
            KeyWeight {
                key: PublicKey::new_from_str(
                    "PUB_R1_82ua5qburg82c9eWY1qZVNUAAD6VPHsTMoPMGDrk7s4BQgxEoc",
                )
                .unwrap(),
                weight: 2,
            },
        ],
        waits: vec![WaitWeight {
            wait_sec: 10,
            weight: 1,
        }],
        accounts: vec![],
    };

    let auth_packed = bytes_to_hex(&Encoder::pack(&auth));
    assert_eq!(auth_packed, data);
    /*
    assert.ok(auth.hasPermission('EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin'))
    assert.ok(
        auth.hasPermission('PUB_R1_82ua5qburg82c9eWY1qZVNUAAD6VPHsTMoPMGDrk7s4BQgxEoc', true)
    )
    assert.ok(!auth.hasPermission('PUB_R1_82ua5qburg82c9eWY1qZVNUAAD6VPHsTMoPMGDrk7s4BQgxEoc'))
    assert.ok(!auth.hasPermission('PUB_K1_6E45rq9ZhnvnWNTNEEexpM8V8rqCjggUWHXJBurkVQSnEyCHQ9'))
    assert.ok(
        !auth.hasPermission('PUB_K1_6E45rq9ZhnvnWNTNEEexpM8V8rqCjggUWHXJBurkVQSnEyCHQ9', true)
    )
    */
}
/*

    test('packed transaction', function () {
        // uncompressed packed transaction
        const uncompressed = PackedTransaction.from({
            packed_trx:
                '34b6c664cb1b3056b588000000000190e2a51c5f25af590000000000e94c4402308db3ee1bf7a88900000000a8ed3232e04c9bae3b75a88900000000a8ed323210e04c9bae3b75a889529e9d0f0001000000',
        })
        assert.instanceOf(uncompressed.getTransaction(), Transaction)

        // zlib compressed packed transation
        const compressedString =
            '78dacb3d782c659f64208be036062060345879fad9aa256213401c8605cb2633322c79c8c0e8bd651e88bfe2ad9191204c80e36d735716638b77330300024516b4'

        // This is a compressed transaction and should throw since it cannot be read without a compression flag
        const compressedError = PackedTransaction.from({
            packed_trx: compressedString,
        })
        assert.throws(() => compressedError.getTransaction())

        // This is a compressed transaction and should succeed since it has a compression flag
        const compressedSuccess = PackedTransaction.from({
            compression: 1,
            packed_trx: compressedString,
        })
        assert.instanceOf(compressedSuccess.getTransaction(), Transaction)
    })
})

 */
