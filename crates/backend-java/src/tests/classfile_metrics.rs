//! Minimal dependency-free test reader for Java 21 class-file capacity metrics.

#[derive(Debug, Default)]
pub(super) struct Method {
    pub name: String,
    pub code: usize,
    pub locals: usize,
    pub stack: usize,
    pub exceptions: usize,
    pub frames: usize,
}

#[derive(Debug, Default)]
pub(super) struct Class {
    pub pool: usize,
    pub fields: usize,
    pub methods: Vec<Method>,
    pub bootstraps: usize,
    pub bootstrap_arguments: usize,
    pub metadata: usize,
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> &'a [u8] {
        let end = self
            .offset
            .checked_add(size)
            .expect("class offset overflow");
        let bytes = self
            .bytes
            .get(self.offset..end)
            .expect("truncated class file");
        self.offset = end;
        bytes
    }
    fn u1(&mut self) -> usize {
        usize::from(self.take(1)[0])
    }
    fn u2(&mut self) -> usize {
        usize::from(u16::from_be_bytes(self.take(2).try_into().unwrap()))
    }
    fn u4(&mut self) -> usize {
        u32::from_be_bytes(self.take(4).try_into().unwrap()) as usize
    }
    fn attributes(&mut self, names: &[String], mut visit: impl FnMut(&str, &mut Reader<'_>)) {
        let count = self.u2();
        for _ in 0..count {
            let name = &names[self.u2()];
            let size = self.u4();
            let mut payload = Reader {
                bytes: self.take(size),
                offset: 0,
            };
            visit(name, &mut payload);
        }
    }
}

pub(super) fn read(path: &std::path::Path) -> Class {
    let bytes = std::fs::read(path).expect("read native class file");
    let mut reader = Reader {
        bytes: &bytes,
        offset: 0,
    };
    assert_eq!(reader.u4(), 0xcafebabe);
    reader.u2();
    assert_eq!(reader.u2(), 65, "oracle must emit Java 21 class files");
    let count = reader.u2();
    let mut names = vec![String::new(); count];
    let mut index = 1;
    while index < count {
        match reader.u1() {
            1 => {
                let size = reader.u2();
                names[index] = String::from_utf8_lossy(reader.take(size)).into_owned();
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => {
                reader.take(4);
            }
            5 | 6 => {
                reader.take(8);
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => {
                reader.take(2);
            }
            15 => {
                reader.take(3);
            }
            tag => panic!("unknown constant-pool tag {tag}"),
        }
        index += 1;
    }
    reader.take(6); // Access flags, this class, super class.
    let interfaces = reader.u2();
    reader.take(interfaces * 2);
    let fields = reader.u2();
    for _ in 0..fields {
        reader.take(6);
        reader.attributes(&names, |_, _| {});
    }
    let mut class = Class {
        pool: count - 1,
        fields,
        ..Class::default()
    };
    let methods = reader.u2();
    for _ in 0..methods {
        reader.u2();
        let name = names[reader.u2()].clone();
        reader.u2();
        let mut method = Method {
            name,
            ..Method::default()
        };
        reader.attributes(&names, |name, body| {
            if name != "Code" {
                return;
            }
            method.stack = body.u2();
            method.locals = body.u2();
            method.code = body.u4();
            body.take(method.code);
            method.exceptions = body.u2();
            body.take(method.exceptions * 8);
            body.attributes(&names, |name, frames| {
                if name == "StackMapTable" {
                    method.frames = frames.u2();
                }
            });
        });
        class.methods.push(method);
    }
    reader.attributes(&names, |name, body| match name {
        "BootstrapMethods" => {
            class.bootstraps = body.u2();
            for _ in 0..class.bootstraps {
                body.u2();
                let arguments = body.u2();
                class.bootstrap_arguments = class.bootstrap_arguments.max(arguments);
                body.take(arguments * 2);
            }
        }
        "InnerClasses" | "NestMembers" => class.metadata = class.metadata.max(body.u2()),
        _ => {}
    });
    assert_eq!(reader.offset, bytes.len(), "unread class-file tail");
    class
}
