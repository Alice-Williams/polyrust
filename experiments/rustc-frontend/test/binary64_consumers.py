"""Handwritten independent consumers. Bit inspection never enters generated code."""
from binary64_oracle import VALUES


def java_consumer(calls):
    prefix = """public final class Consumer {
    static void observe(double value) {
        if (Double.isNaN(value)) System.out.println("nan");
        else System.out.printf("%016x%n", Double.doubleToRawLongBits(value));
        System.err.println();
    }
    public static void main(String[] args) {
"""
    values = ",".join(f"0x{bits:016x}L" for bits in VALUES)
    literal, transport, comparison = calls
    return (prefix + "\n".join(f"observe({call});" for call in literal)
            + "\nlong[] values = {" + values + "};\n"
            + "for(long bits: values) { double a = Double.longBitsToDouble(bits);\n"
            + "\n".join(f"observe({call});" for call in transport) + "}\n"
            + "for(long first: values) { for(long second: values) {\n"
            + "double a = Double.longBitsToDouble(first), b = Double.longBitsToDouble(second);\n"
            + "\n".join(f"System.out.println({call} ? 1 : 0); System.err.println();" for call in comparison)
            + "} } } }\n")


def c_consumer(calls, headers):
    prefix = """#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <float.h>
static void observe(double value) {
    uint64_t bits;
    memcpy(&bits, &value, sizeof(bits));
    if ((bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000))
        (void)puts("nan");
    else (void)printf("%016" PRIx64 "\\n", bits);
    (void)fputc('\\n', stderr);
}
static double decode(uint64_t bits) {
    double value;
    memcpy(&value, &bits, sizeof(value));
    return value;
}
"""
    literal, transport, comparison = calls
    values = ",".join(f"UINT64_C(0x{bits:016x})" for bits in VALUES)
    return (prefix + headers + "\nint main(void) {\n"
            + "\n".join(f"observe({call});" for call in literal)
            + "\nconst uint64_t values[] = {" + values + "};\n"
            + f"for(int i=0;i<{len(VALUES)};i++) {{ double a=decode(values[i]);\n"
            + "\n".join(f"observe({call});" for call in transport) + "}\n"
            + f"for(int i=0;i<{len(VALUES)};i++) {{ for(int j=0;j<{len(VALUES)};j++) {{\n"
            + "double a=decode(values[i]), b=decode(values[j]);\n"
            + "\n".join(f'(void)printf("%d\\n", {call}); (void)fputc(\'\\n\', stderr);' for call in comparison)
            + "} } return 0; }\n")
