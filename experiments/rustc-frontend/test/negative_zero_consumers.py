"""Independent external streaming clients with per-case trace boundaries."""


def java_consumer(member):
    return """public final class Consumer {
private Consumer() {}
public static void main(String[] args) throws java.io.IOException {
    var reader = new java.io.BufferedReader(new java.io.InputStreamReader(
        System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        double value = Double.longBitsToDouble(Long.parseUnsignedLong(line, 16));
""" + f'System.out.println({member}(value) ? 1 : 0);\nSystem.err.print("|");\n' + "} } }\n"


def c_consumer(header, member):
    return f'#include "{header}"\n' + """#include <inttypes.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    uint64_t bits;
    while (scanf("%" SCNx64, &bits) == 1) {
        double value;
        (void)memcpy(&value, &bits, sizeof value);
""" + f'(void)printf("%d\\n", {member}(value) ? 1 : 0);\n(void)fputc(\'|\', stderr);\n' + "} return 0; }\n"
