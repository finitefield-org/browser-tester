package script

type ValueKind string

const (
	ValueKindUndefined  ValueKind = "undefined"
	ValueKindString     ValueKind = "string"
	ValueKindBool       ValueKind = "bool"
	ValueKindNumber     ValueKind = "number"
	ValueKindInvocation ValueKind = "invocation"
)

type Value struct {
	Kind       ValueKind
	String     string
	Bool       bool
	Number     float64
	Invocation string
}

func UndefinedValue() Value {
	return Value{Kind: ValueKindUndefined}
}

func StringValue(value string) Value {
	return Value{
		Kind:   ValueKindString,
		String: value,
	}
}

func BoolValue(value bool) Value {
	return Value{
		Kind: ValueKindBool,
		Bool: value,
	}
}

func NumberValue(value float64) Value {
	return Value{
		Kind:   ValueKindNumber,
		Number: value,
	}
}

func InvocationValue(source string) Value {
	return Value{
		Kind:       ValueKindInvocation,
		Invocation: source,
	}
}
