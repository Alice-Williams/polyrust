# Rust f64::is_nan in C17

- Status: implemented and verified for standard inherent f64::is_nan
- Contract: [shared](../../rust-floating-nan.md)

CFloatingNaN consumes the private canonical NaNInput and validates its original
Reader context. Lower and materialize the source receiver once, require exact
CScalarType::F64, then build CBinaryOperator::NotEqual from two reads of that
same local. The C comparison has Int result; apply the existing typed numeric
conversion to Bool. Calls remain sequenced full-expression initializers.
No target grammar or certification relaxation is intended.

NaN is unordered with itself under the admitted IEC 60559 binary64 profile;
equality is false and inequality true
([N1570 6.5.9 and 7.12.14](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)).
Floating exception flags are not part of the admitted source interface.
The default nontrapping environment and no-fast-math requirements are unchanged.

Use the existing C type/identifier/dependency/certification constructors and
ordinary renderer. No math.h import or copied runtime is emitted. Native tests
must compile original owner units separately using pinned GCC14 and Zig at
O0/O2 with C17 strict warnings. Integer-bit classification is the independent
oracle; test copies establish value and exactly-once receiver observability.
