extern crate concrust;

pub use self::concrust::map::ConcurrentHashMap;

describe! hash_map_tests {

    before_each {
        let mut map = ConcurrentHashMap::new();
    }

    it "should create new empty map" {
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    it "should create a new map with default capacity" {
        assert_eq!(map.capacity(), 16);
    }

    it "should have capacity that is always highest power of two" {
        let map = ConcurrentHashMap::with_capacity(6);
        assert_eq!(map.capacity(), 8);
        let map = ConcurrentHashMap::with_capacity(10);
        assert_eq!(map.capacity(), 16);
        let map = ConcurrentHashMap::with_capacity(100);
        assert_eq!(map.capacity(), 128);
    }

    it "should increase size when insert into map" {
        map.insert(1, 1);
        assert!(!map.is_empty());
    }
}
