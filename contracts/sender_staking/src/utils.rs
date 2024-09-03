use near_sdk::Timestamp;

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub mod u64_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub fn to_nano(sec: u32) -> Timestamp {
    Timestamp::from(sec) * 10u64.pow(9)
}

pub fn nano_to_sec(nano: u64) -> u64 {
    (nano / 10u64.pow(9)) as u64
}

pub fn as_u128(arr: &[u8]) -> u128 {
    ((arr[0] as u128) << 0) +
    ((arr[1] as u128) << 8) +
    ((arr[2] as u128) << 16) +
    ((arr[3] as u128) << 24) +
    ((arr[4] as u128) << 32) +
    ((arr[5] as u128) << 40) +
    ((arr[6] as u128) << 48) +
    ((arr[7] as u128) << 56) +
    ((arr[8] as u128) << 64) +
    ((arr[9] as u128) << 72) +
    ((arr[10] as u128) << 80) +
    ((arr[11] as u128) << 88) +
    ((arr[12] as u128) << 96) +
    ((arr[13] as u128) << 104) +
    ((arr[14] as u128) << 112) +
    ((arr[15] as u128) << 120)
}
