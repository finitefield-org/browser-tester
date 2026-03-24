package script

import "fmt"

type ErrorKind string

const (
	ErrorKindParse       ErrorKind = "parse"
	ErrorKindRuntime     ErrorKind = "runtime"
	ErrorKindHost        ErrorKind = "host"
	ErrorKindUnsupported ErrorKind = "unsupported"
)

type Error struct {
	Kind    ErrorKind
	Message string
}

func NewError(kind ErrorKind, message string) Error {
	return Error{
		Kind:    kind,
		Message: message,
	}
}

func (e Error) Error() string {
	switch {
	case e.Kind == "" && e.Message == "":
		return "script error"
	case e.Kind == "":
		return e.Message
	case e.Message == "":
		return string(e.Kind)
	default:
		return fmt.Sprintf("%s: %s", e.Kind, e.Message)
	}
}
