"""External binary-protocol consumers, never emitted as part of a package."""
from character_source_inventory import expressions


def c_client(evidence):
    _, _, apis, _, _, _, _ = evidence
    prefix, rows = expressions(evidence, "c")
    includes = "".join('#include "' + api["header"] + '"\n' for api in apis.values())
    source = r"""
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
static uint32_t read32(const unsigned char *p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}
static int write32(uint32_t value) {
    unsigned char bytes[4];
    for (unsigned int i = 0; i < 4; ++i) bytes[i] = (unsigned char)(value >> (8 * i));
    return fwrite(bytes, 1, 4, stdout) == 4;
}
static bool scalar(uint32_t value) {
    return value <= UINT32_C(0x10ffff) && !(value >= UINT32_C(0xd800) && value <= UINT32_C(0xdfff));
}
int main(void) {
PREFIX
    unsigned char bytes[12];
    size_t length;
    while ((length = fread(bytes, 1, sizeof bytes, stdin)) != 0) {
        if (length != sizeof bytes) return 2;
        uint32_t left = read32(bytes), right = read32(bytes + 4), marker_bits = read32(bytes + 8);
        int32_t marker;
        memcpy(&marker, &marker_bits, sizeof marker);
        if (!scalar(left) || !scalar(right)) return 3;
ROWS
    }
    return ferror(stdin) != 0 || fflush(stdout) != 0;
}
"""
    # Separate full expressions guarantee consumer order; generated code must
    # independently preserve each function's internal Rust evaluation order.
    return includes + source.replace("PREFIX", "\n".join(
        f"    if (!write32((uint32_t)({expr}))) return 4;" for expr in prefix)).replace(
            "ROWS", "\n".join(f"        if (!write32((uint32_t)({expr}))) return 4;" for expr in rows))


def java_client(evidence):
    prefix, rows = expressions(evidence, "java")
    source = r"""public final class Consumer {
private Consumer() {}
private static int read32(byte[] b, int n) {
    return (b[n] & 255) | ((b[n + 1] & 255) << 8) | ((b[n + 2] & 255) << 16) | ((b[n + 3] & 255) << 24);
}
private static boolean scalar(int value) {
    return value >= 0 && value <= 0x10ffff && !(value >= 0xd800 && value <= 0xdfff);
}
public static void main(String[] args) throws java.io.IOException {
    java.io.DataOutputStream output = new java.io.DataOutputStream(new java.io.BufferedOutputStream(System.out));
    java.io.BufferedInputStream input = new java.io.BufferedInputStream(System.in);
PREFIX
    byte[] bytes = new byte[12];
    for (;;) {
        int count = input.readNBytes(bytes, 0, bytes.length);
        if (count == 0) break;
        if (count != bytes.length) throw new java.io.IOException("truncated packet");
        int left = read32(bytes, 0), right = read32(bytes, 4), marker = read32(bytes, 8);
        if (!scalar(left) || !scalar(right)) throw new java.io.IOException("invalid scalar input");
ROWS
    }
    output.flush();
}
}
"""
    return source.replace("PREFIX", "\n".join(
        f"    output.writeInt(Integer.reverseBytes({expr}));" for expr in prefix)).replace(
            "ROWS", "\n".join(f"        output.writeInt(Integer.reverseBytes({expr}));" for expr in rows))
