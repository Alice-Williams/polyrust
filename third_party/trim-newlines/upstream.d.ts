type Newline =
	| '\u{A}'
	| '\u{D}'
	;

type TrimStart<S extends string> = S extends `${Newline}${infer R}` ? TrimStart<R> : S;

type TrimEnd<S extends string> = S extends `${infer R}${Newline}` ? TrimEnd<R> : S;

export type Trim<S extends string> = TrimStart<TrimEnd<S>>;

export function trimNewlines<S extends string>(string: S): Trim<S>;

export function trimNewlinesStart<S extends string>(string: S): TrimStart<S>;

export function trimNewlinesEnd<S extends string>(string: S): TrimEnd<S>;
