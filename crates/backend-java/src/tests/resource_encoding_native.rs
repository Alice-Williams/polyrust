//! In-memory javac avoids filesystem filename limits hiding JVM UTF8 limits.

use super::totality_oracle::CompiledPackage;
use portable_build::{portable_name, typed_program};

#[test]
fn java21_in_memory_encoding_and_synthesis_boundaries() {
    let program = typed_program(portable_name!("encoding_capacity"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    CompiledPackage::new(&manifest, "encoding-capacity").consumer(CONTROLS);
}

const CONTROLS: &str = r#"
package org.polyrust.consumer;
public final class Consumer {
    private Consumer() {}
    private static final class Source extends javax.tools.SimpleJavaFileObject {
        private final String source;
        Source(String source) {
            super(java.net.URI.create("string:///p/Fixture.java"), javax.tools.JavaFileObject.Kind.SOURCE);
            this.source = source;
        }
        @Override public CharSequence getCharContent(boolean ignore) { return source; }
    }
    private static void check(String body, boolean expected, String diagnostic) throws java.io.IOException {
        javax.tools.JavaCompiler compiler = javax.tools.ToolProvider.getSystemJavaCompiler();
        javax.tools.DiagnosticCollector<javax.tools.JavaFileObject> diagnostics = new javax.tools.DiagnosticCollector<>();
        java.util.Map<String, byte[]> outputs = new java.util.LinkedHashMap<>();
        try (javax.tools.StandardJavaFileManager standard = compiler.getStandardFileManager(diagnostics, null, null);
             javax.tools.JavaFileManager memory = new javax.tools.ForwardingJavaFileManager<javax.tools.StandardJavaFileManager>(standard) {
                 @Override public javax.tools.JavaFileObject getJavaFileForOutput(javax.tools.JavaFileManager.Location location, String name, javax.tools.JavaFileObject.Kind kind, javax.tools.FileObject sibling) {
                     return new javax.tools.SimpleJavaFileObject(java.net.URI.create("memory:///" + name.replace('.', '/') + kind.extension), kind) {
                         @Override public java.io.OutputStream openOutputStream() {
                             return new java.io.ByteArrayOutputStream() {
                                 @Override public void close() throws java.io.IOException { super.close(); outputs.put(name, toByteArray()); }
                             };
                         }
                     };
                 }
             }) {
            String source = "package p; public final class Fixture { private Fixture() {} " + body + " }";
            boolean accepted = compiler.getTask(null, memory, diagnostics, java.util.List.of("--release", "21", "-Xlint:all", "-Werror"), null, java.util.List.of(new Source(source))).call();
            String messages = diagnostics.getDiagnostics().stream().map(d -> d.getMessage(java.util.Locale.ROOT)).collect(java.util.stream.Collectors.joining("\n"));
            if (accepted) {
                ClassLoader loader = new ClassLoader(Consumer.class.getClassLoader()) {
                    @Override protected Class<?> findClass(String name) throws ClassNotFoundException {
                        byte[] bytes = outputs.get(name);
                        if (bytes == null) throw new ClassNotFoundException(name);
                        return defineClass(name, bytes, 0, bytes.length);
                    }
                };
                try {
                    for (String name : outputs.keySet()) {
                        Class<?> loaded = Class.forName(name, false, loader);
                        loaded.getDeclaredMethods();
                        loaded.getDeclaredConstructors();
                    }
                } catch (LinkageError failure) {
                    accepted = false;
                    messages += failure.toString();
                } catch (ClassNotFoundException failure) { throw new AssertionError(failure); }
            }
            if (accepted != expected || (!expected && !messages.contains(diagnostic))) {
                throw new AssertionError("expected=" + expected + ", actual=" + accepted + ": " + messages.substring(0, Math.min(messages.length(), 500)));
            }
        }
    }
    private static String parameters(int count) {
        return java.util.stream.IntStream.range(0, count).mapToObj(i -> "int p" + i).collect(java.util.stream.Collectors.joining(","));
    }
    private static String arguments(int count) { return java.util.Collections.nCopies(count, "0").stream().collect(java.util.stream.Collectors.joining(",")); }
    public static void main(String[] args) throws java.io.IOException {
        for (int dimensions : new int[] {255, 256}) {
            check("public static int" + "[]".repeat(dimensions) + " array() { return null; }", dimensions == 255, "too many dimensions");
        }
        for (int slots : new int[] {252, 253}) {
            check("public enum E { VALUE(" + arguments(slots) + "); private E(" + parameters(slots) + ") {} }", slots == 252, "many");
        }
        for (int slots : new int[] {253, 254}) {
            check("public final class Inner { public Inner(" + parameters(slots) + ") {} }", slots == 253, "many");
        }
        String prefix = "p.Fixture$";
        for (int length : new int[] {65535, 65536}) {
            String name = "A".repeat(length - prefix.length());
            check("public static final class " + name + " {}", length == 65535, "too long");
        }
        for (int length : new int[] {65530, 65531}) {
            String name = "A".repeat(length - prefix.length());
            check("public static final class " + name + " { public final class B {} }", length == 65530, "too long");
        }
        String longType = "A".repeat(40000);
        for (int count : new int[] {1, 2}) {
            String parameters = java.util.stream.IntStream.range(0, count).mapToObj(i -> longType + " p" + i).collect(java.util.stream.Collectors.joining(","));
            check("public static final class " + longType + " {} public static void use(" + parameters + ") {}", count == 1, "too long");
        }
        String first = "A".repeat(40000);
        String second = "B".repeat(40000);
        String interfaces = "public interface " + first + " {} public interface " + second + " {} ";
        check(interfaces + "public static final class C implements " + first + "," + second + " {}", true, "");
        check(interfaces + "public static final class C<T> implements " + first + "," + second + " {}", false, "too long");
    }
}
"#;
