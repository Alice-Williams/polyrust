//! Independent native value and trace oracle; never input to the production emitter.
use super::package::function;
use portable_backend_c::dialect::CDependencyApi;

pub fn driver(
    owner: &CDependencyApi,
    a: &CDependencyApi,
    b: &CDependencyApi,
    consumer: &CDependencyApi,
    backward: &CDependencyApi,
) -> String {
    let proof = owner.structs().next().unwrap();
    let ty = proof.symbol().as_str();
    let tag = proof.member_name(&proof.members()[0]).unwrap().as_str();
    let payload = proof.member_name(&proof.members()[1]).unwrap().as_str();
    let name = |api, role| function(api, role).symbol().as_str().to_owned();
    let ac = name(a, "construct");
    let af = name(a, "forward");
    let bc = name(b, "construct");
    let bf = name(b, "forward");
    let at = name(a, "tag");
    let av = name(a, "payload");
    let bt = name(b, "tag");
    let bv = name(b, "payload");
    let compose = name(consumer, "compose");
    let back_compose = name(backward, "compose");
    format!(
        r#"#include "{ah}"
#include "{bh}"
#include "{ch}"
#include "{ch}"
#include "{dh}"
#include <stdio.h>
static unsigned trace;
extern struct {ty} actual_{ac}(_Bool success, int32_t value);
extern struct {ty} actual_{bc}(_Bool success, int32_t value);
extern struct {ty} actual_{af}(struct {ty} value);
extern struct {ty} actual_{bf}(struct {ty} value);
struct {ty} {ac}(_Bool success, int32_t value) {{
    trace = trace * 10u + 1u; return actual_{ac}(success, value);
}}
struct {ty} {af}(struct {ty} value) {{
    trace = trace * 10u + 1u; return actual_{af}(value);
}}
struct {ty} {bc}(_Bool success, int32_t value) {{
    trace = trace * 10u + 2u; return actual_{bc}(success, value);
}}
struct {ty} {bf}(struct {ty} value) {{
    trace = trace * 10u + 2u; return actual_{bf}(value);
}}
static int check(_Bool success, int32_t value) {{
    struct {ty} first = {ac}(success, value);
    struct {ty} second = {bc}(success, value);
    struct {ty} a_copy = {af}(second);
    struct {ty} b_copy = {bf}(first);
    const struct {ty} copies[] = {{first, second, a_copy, b_copy}};
    for (size_t i = 0; i < sizeof copies / sizeof copies[0]; ++i) {{
        if (copies[i].{tag} != success) return 11;
        if (copies[i].{payload} != value) return 12;
        if ({at}(copies[i]) != success || {bt}(copies[i]) != success) return 14;
        if ({av}(copies[i]) != value || {bv}(copies[i]) != value) return 15;
    }}
    trace = 0;
    struct {ty} crossed = {compose}(success, value);
    if (crossed.{tag} != success) return 11;
    if (crossed.{payload} != value) return 12;
    if (trace != 12u) return 13;
    trace = 0;
    struct {ty} back_crossed = {back_compose}(success, value);
    if (back_crossed.{tag} != success) return 11;
    if (back_crossed.{payload} != value) return 12;
    if (trace != 21u) return 13;
    return 0;
}}
int main(void) {{
    unsigned count = 0;
    /* Explicit independent version-2 wire codes, not production enum iteration. */
    const int32_t errors[] = {{0, 1, 2, 3, 4, 5}};
    for (size_t i = 0; i < sizeof errors / sizeof errors[0]; ++i) {{
        int status = check(0, errors[i]); if (status) return status; ++count;
    }}
    for (int32_t value = -4096; value <= 4096; ++value) {{
        int status = check(1, value); if (status) return status; ++count;
    }}
    const int32_t edges[] = {{INT32_MIN, INT32_MIN + 1, INT32_MAX - 1, INT32_MAX}};
    for (size_t i = 0; i < sizeof edges / sizeof edges[0]; ++i) {{
        int status = check(1, edges[i]); if (status) return status; ++count;
    }}
    if (count != 8203u) return 16;
    puts("8203 canonical states and ordered cross-producer calls");
    return 0;
}}
"#,
        ah = a.public_header().include_path(),
        bh = b.public_header().include_path(),
        ch = consumer.public_header().include_path(),
        dh = backward.public_header().include_path()
    )
}
