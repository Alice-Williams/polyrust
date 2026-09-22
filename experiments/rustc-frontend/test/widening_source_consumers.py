"""Handwritten one-input, signed-wide-output external consumers."""


def consumer(calls, java, headers=""):
    if java:
        return ('public final class Consumer { private Consumer() {} public static void main(String[] args) throws java.io.IOException {\n'
                'var reader=new java.io.BufferedReader(new java.io.InputStreamReader(System.in,java.nio.charset.StandardCharsets.UTF_8));\n'
                'String line; while((line=reader.readLine())!=null) { int value=Integer.parseInt(line);\n'
                + "".join(f"System.out.println({name}(value));\n" for name in calls) + '} } }\n')
    return ('#include <inttypes.h>\n#include <stdio.h>\n' + headers
            + 'int main(void) { int32_t value; while(scanf("%" SCNd32,&value)==1) {\n'
            + "".join(f'printf("%" PRId64 "\\n", {name}(value));\n' for name in calls)
            + '} return 0; }\n')
