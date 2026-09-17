"""Handwritten bit-observing clients; conversions are never generated library code."""
from binary64_oracle import VALUES


def java_consumer(literals, calls):
    values = ",".join(f"0x{bits:016x}L" for bits in VALUES)
    return ("""public final class Consumer {
static void observe(double value) {
    if (Double.isNaN(value)) System.out.println("nan");
    else System.out.printf("%016x%n", Double.doubleToRawLongBits(value));
}
public static void main(String[] args) {
""" + "\n".join(f"observe({call});" for call in literals)
            + "\nlong[] values = {" + values + "};\n"
            + "for(long bits: values) { double a = Double.longBitsToDouble(bits);\n"
            + "\n".join(f"observe({call});" for call in calls) + "} } }\n")


def c_consumer(literals, calls, headers):
    values = ",".join(f"UINT64_C(0x{bits:016x})" for bits in VALUES)
    return ("""#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <fenv.h>
static void observe(double value) {
    uint64_t bits;
    memcpy(&bits, &value, sizeof(bits));
    if ((bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000))
        (void)puts("nan");
    else (void)printf("%016" PRIx64 "\\n", bits);
}
static double decode(uint64_t bits) {
    double value; memcpy(&value, &bits, sizeof(value)); return value;
}
""" + headers + "\nint main(void) {\nif (fegetround() != FE_TONEAREST) return 2;\n"
            + "\n".join(f"observe({call});" for call in literals)
            + "\nconst uint64_t values[] = {" + values + "};\n"
            + f"for(int i=0;i<{len(VALUES)};i++) {{ double a=decode(values[i]);\n"
            + "\n".join(f"observe({call});" for call in calls) + "} return 0; }\n")
