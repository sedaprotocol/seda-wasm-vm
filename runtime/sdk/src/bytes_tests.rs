use super::*;

#[test]
fn bytes_basic() {
    let vec = vec![0, 1, 2, 3];
    let bytes = vec.clone().to_bytes();

    assert_eq!(&vec, &*bytes);
    assert_eq!(vec, bytes.clone().eject());
    assert_eq!(bytes.clone(), bytes.clone().to_bytes());

    assert_eq!(vec, Vec::<u8>::from_bytes(&bytes).unwrap());
    assert_eq!(vec, Vec::<u8>::from_bytes_vec(bytes.clone().eject()).unwrap());

    let ser = serde_json::to_string(&bytes).unwrap();
    let deser = serde_json::from_str::<Bytes>(&ser).unwrap();
    assert_eq!(bytes, deser);
}

#[test]
fn bytes_string() {
    let string = "hello world".to_string();
    let bytes = string.clone().to_bytes();

    assert_eq!(string.as_bytes(), &*bytes);

    assert_eq!(string, String::from_bytes(&bytes).unwrap());
    assert_eq!(string, String::from_bytes_vec(bytes.eject()).unwrap());
}

#[test]
fn bytes_str() {
    let str = "hello world";
    let bytes = str.to_bytes();

    assert_eq!(str.as_bytes(), &*bytes);
}

#[test]
fn bytes_bool() {
    let bytes = true.to_bytes();
    assert_eq!(vec![1], *bytes);

    assert!(bool::from_bytes(&bytes).unwrap());
    assert!(bool::from_bytes_vec(bytes.eject()).unwrap());

    let bytes = false.to_bytes();
    assert_eq!(vec![0], *bytes);

    assert!(!bool::from_bytes(&bytes).unwrap());
    assert!(!bool::from_bytes_vec(bytes.eject()).unwrap());

    let bytes = vec![0, 1].to_bytes();
    assert!(bool::from_bytes(&bytes).is_err());
    assert!(bool::from_bytes_vec(bytes.eject()).is_err());

    let bytes = vec![2].to_bytes();
    assert!(bool::from_bytes(&bytes).is_err());
    assert!(bool::from_bytes_vec(bytes.eject()).is_err());
}

#[test]
fn bytes_zero_size() {
    let bytes = ().to_bytes();
    assert_eq!(Vec::<u8>::new(), *bytes);
}

macro_rules! test_bytes_impls_le_bytes {
    ($type:ty, $num_bytes:expr, $test_value:expr) => {
        ::paste::paste! {
          #[test]
          fn [<bytes_$type>]() {
            let value: $type = $test_value;
            let bytes = value.to_bytes();
            assert_eq!($num_bytes, bytes.len());
            assert_eq!(value.to_le_bytes(), &*bytes);
            assert_eq!(value, <$type>::from_bytes(&bytes).unwrap());
            assert_eq!(value, <$type>::from_bytes_vec(bytes.eject()).unwrap());
          }
        }
    };
}

test_bytes_impls_le_bytes!(u8, 1, 0x12);
test_bytes_impls_le_bytes!(u32, 4, 0x12345678);
test_bytes_impls_le_bytes!(u64, 8, 0x123456789abcdef0);
test_bytes_impls_le_bytes!(u128, 16, 0x123456789abcdef0123456789abcdef0);
test_bytes_impls_le_bytes!(i8, 1, 0x12);
test_bytes_impls_le_bytes!(i32, 4, 0x12345678);
test_bytes_impls_le_bytes!(i64, 8, 0x123456789abcdef0);
test_bytes_impls_le_bytes!(i128, 16, 0x123456789abcdef0123456789abcdef0);
test_bytes_impls_le_bytes!(f32, 4, 10.8347);
test_bytes_impls_le_bytes!(f64, 8, 10.8347);
