"""Handwritten streaming clients for exact bit/category observations."""


def java_consumer(calls):
    return """public final class Consumer {
private Consumer() {}
private static void observe(double value) {
    if (Double.isNaN(value)) { System.out.println("nan"); }
    else { System.out.printf(java.util.Locale.ROOT, "%016x%n", Double.doubleToRawLongBits(value)); }
}
public static void main(String[] args) throws java.io.IOException {
    java.io.BufferedReader reader = new java.io.BufferedReader(
        new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        String[] parts = line.split(" ");
        double left = Double.longBitsToDouble(Long.parseUnsignedLong(parts[0], 16));
        double right = Double.longBitsToDouble(Long.parseUnsignedLong(parts[1], 16));
""" + "".join("observe(" + call + ");\n" for call in calls) + "} } }\n"


def c_consumer(calls, headers):
    return headers + """#include <inttypes.h>
#include <stdio.h>
#include <string.h>
static void observe(double value) {
    uint64_t bits;
    (void)memcpy(&bits, &value, sizeof bits);
    if ((bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000)) {
        (void)puts("nan");
    } else {
        (void)printf("%016" PRIx64 "\\n", bits);
    }
}
int main(void) {
    uint64_t left_bits, right_bits;
    while (scanf("%" SCNx64 " %" SCNx64, &left_bits, &right_bits) == 2) {
        double left, right;
        (void)memcpy(&left, &left_bits, sizeof left);
        (void)memcpy(&right, &right_bits, sizeof right);
""" + "".join("observe(" + call + ");\n" for call in calls) + "} return 0; }\n"
