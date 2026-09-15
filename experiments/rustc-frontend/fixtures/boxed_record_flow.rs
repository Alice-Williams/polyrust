//! Accepted and rejected Box<Record> source/producer/cleanup shapes.
pub struct Record {
    number: i32,
    enabled: bool,
}
pub fn number(value: i32, flag: bool) -> i32 {
    let record = Record {
        number: value,
        enabled: flag,
    };
    let owner = Box::new(record);
    (*owner).number
}
pub fn boolean(value: i32, flag: bool) -> bool {
    let record = Record {
        enabled: flag,
        number: value,
    };
    let owner = Box::new(record);
    (*owner).enabled
}
pub fn moved(value: i32, flag: bool) -> i32 {
    let record = Record {
        enabled: flag,
        number: value,
    };
    let owner = Box::new(record);
    let moved = owner;
    let final_owner = moved;
    (*final_owner).number
}
pub fn explicit(value: i32, flag: bool) -> i32 {
    let record = Record {
        number: value,
        enabled: flag,
    };
    let owner = Box::new(record);
    return (*owner).number;
}
pub fn implicit(value: i32, flag: bool) -> i32 {
    let record = Record {
        number: value,
        enabled: flag,
    };
    let owner = Box::new(record);
    owner.number
}
#[derive(Clone, Copy)]
pub struct CopyRecord {
    number: i32,
    enabled: bool,
}
pub fn copied(value: i32, flag: bool) -> i32 {
    let record = CopyRecord {
        number: value,
        enabled: flag,
    };
    let owner = Box::new(record);
    (*owner).number
}
pub fn tail(value: i32) -> i32 {
    let owner = Box::new(value);
    *owner
}
type Alias = Record;
use std::boxed::Box as Heap;
pub fn alias(value: i32, flag: bool) -> i32 {
    let record = Alias {
        number: value,
        enabled: flag,
    };
    let owner = Heap::new(record);
    (*owner).number
}
pub fn shadow(value: i32, flag: bool) -> i32 {
    let value = Record {
        number: value,
        enabled: flag,
    };
    let value = Box::new(value);
    let value = value;
    (*value).number
}
pub fn same_type(unused: i32, value: i32, flag: bool) -> i32 {
    let record = Record {
        number: value,
        enabled: flag,
    };
    let owner = Box::new(record);
    (*owner).number
}
pub mod left {
    pub struct Record {
        pub number: i32,
        pub enabled: bool,
    }
    pub fn same(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
}
pub mod right {
    pub struct Record {
        pub number: i32,
        pub enabled: bool,
    }
    pub fn same(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
}
pub mod rejected {
    use super::Record;
    pub fn make(record: Record) -> Box<Record> {
        Box::new(record)
    }
    pub fn wrapper(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = make(record);
        (*owner).number
    }
    pub fn constant(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: 1,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
    pub fn expression(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value + 1,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
    pub fn mutable(value: i32, flag: bool) -> i32 {
        let mut record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
    pub fn mutation(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let mut owner = Box::new(record);
        owner.number = 4;
        (*owner).number
    }
    pub fn borrowed(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        let reference = &owner;
        (*owner).number
    }
    pub fn direct(value: i32, flag: bool) -> i32 {
        let owner = Box::new(Record {
            number: value,
            enabled: flag,
        });
        (*owner).number
    }
    pub fn nested(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        { (*owner).number }
    }
    pub fn conditional(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        if flag { (*owner).number } else { 0 }
    }
    pub fn update(value: i32, flag: bool) -> i32 {
        let previous = Record {
            number: value,
            enabled: flag,
        };
        let record = Record {
            number: value,
            ..previous
        };
        let owner = Box::new(record);
        (*owner).number
    }
    pub fn parameter(record: Record) -> i32 {
        let owner = Box::new(record);
        (*owner).number
    }
    pub fn extra(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        let extra = Box::new(value);
        (*owner).number
    }
    pub fn indirect(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let constructor = Box::new;
        let owner = constructor(record);
        (*owner).number
    }
    pub fn generic<T>(value: i32, flag: bool) -> i32 {
        let record = Record {
            number: value,
            enabled: flag,
        };
        let owner = Box::new(record);
        (*owner).number
    }
}
