"""Handwritten external clients; widths and ordered entry points are explicit."""


def consumer(calls, java, headers):
    branches = []
    for width in [32, 64]:
        expressions = [f"{name}(left, right)" for name in calls[width]]
        if java:
            ty = "int" if width == 32 else "long"
            parse = "Integer.parseInt" if width == 32 else "Long.parseLong"
            body = f"{ty} left={parse}(parts[1]); {ty} right={parse}(parts[2]);\n"
            body += "".join(f"System.out.println({value});\n" for value in expressions)
        else:
            body = f"int{width}_t left=(int{width}_t)a; int{width}_t right=(int{width}_t)b;\n"
            body += "".join(f'printf("%" PRId{width} "\\n", {value});\n' for value in expressions)
        branches.append(f"if(width=={width}) {{\n{body}}}")
    body = " else ".join(branches)
    if java:
        return ('public final class Consumer { private Consumer() {} public static void main(String[] args) throws java.io.IOException {\n'
                'var reader=new java.io.BufferedReader(new java.io.InputStreamReader(System.in,java.nio.charset.StandardCharsets.UTF_8));\n'
                'String line; while((line=reader.readLine())!=null) { String[] parts=line.split(" "); int width=Integer.parseInt(parts[0]);\n'
                + body + '\n} } }\n')
    return ('#include <inttypes.h>\n#include <stdio.h>\n' + headers
            + 'int main(void) { int width; int64_t a,b; while(scanf("%d %" SCNd64 " %" SCNd64,&width,&a,&b)==3) {\n'
            + body + '\n} return 0; }\n')
