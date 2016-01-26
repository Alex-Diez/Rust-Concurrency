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

    it "should decrease size when remove from map" {
        map.insert(1, 1);
        map.remove(1);
        assert!(map.is_empty());
    }

    it "should remove none if there is no such key" {
        assert_eq!(map.remove(1), None);
    }

    it "should remove inserted value" {
        map.insert(1, 10);
        assert_eq!(map.remove(1), Some(10));
    }

    it "should not remove value that was not inserted into map" {
        map.insert(1, 10);
        let old_size = map.len();
        assert_eq!(map.remove(2), None);
        assert_eq!(map.len(), old_size);
    }

    it "should remove inserted values" {
        map.insert(1, 10);
        map.insert(2, 20);
        map.insert(3, 30);

        assert_eq!(map.remove(1), Some(10));
        assert_eq!(map.remove(2), Some(20));
        assert_eq!(map.remove(3), Some(30));
    }
}
