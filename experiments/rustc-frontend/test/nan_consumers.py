"""Handwritten classification consumers; raw input decoding is test-only."""
from binary64_oracle import VALUES


def java_consumer(literals, calls):
    values = ",".join(f"0x{bits:016x}L" for bits in VALUES)
    return ("public final class Consumer {\n"
            "static void observe(boolean value) { System.out.println(value ? 1 : 0); }\n"
            "public static void main(String[] args) {\n"
            + "\n".join(f"observe({call});" for call in literals)
            + "\nlong[] values = {" + values + "};\n"
            + "for(long bits: values) { double a = Double.longBitsToDouble(bits);\n"
            + "\n".join(f"observe({call});" for call in calls) + "} } }\n")


def c_consumer(literals, calls, headers):
    values = ",".join(f"UINT64_C(0x{bits:016x})" for bits in VALUES)
    return ("""#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fenv.h>
static void observe(_Bool value) { (void)puts(value ? "1" : "0"); }
static double decode(uint64_t bits) {
    double value; memcpy(&value, &bits, sizeof(value)); return value;
}
""" + headers + "\nint main(void) {\nif (fegetround() != FE_TONEAREST) return 2;\n"
            + "\n".join(f"observe({call});" for call in literals)
            + "\nconst uint64_t values[] = {" + values + "};\n"
            + f"for(int i=0;i<{len(VALUES)};i++) {{ double a=decode(values[i]);\n"
            + "\n".join(f"observe({call});" for call in calls) + "} return 0; }\n")
