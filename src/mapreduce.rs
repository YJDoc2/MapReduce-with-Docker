pub trait Mapper<K, V> {
    type Intermediate;
    fn map(key: K, value: V) -> dyn Iterator<Item = Self::Intermediate>;
}

pub trait Reducer<K, V> {
    type Result;
    fn reduce(key: K, mappings: dyn Iterator<Item = V>) -> dyn Iterator<Item = Self::Result>;
}
