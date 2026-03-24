package browsertester

import "fmt"

type ErrorKind string

const (
	ErrorKindHTMLParse     ErrorKind = "html_parse"
	ErrorKindScriptParse   ErrorKind = "script_parse"
	ErrorKindScriptRuntime ErrorKind = "script_runtime"
	ErrorKindSelector      ErrorKind = "selector"
	ErrorKindDOM           ErrorKind = "dom"
	ErrorKindEvent         ErrorKind = "event"
	ErrorKindTimer         ErrorKind = "timer"
	ErrorKindMock          ErrorKind = "mock"
	ErrorKindAssertion     ErrorKind = "assertion"
	ErrorKindUnsupported   ErrorKind = "unsupported"
)

type Error struct {
	Kind    ErrorKind
	Message string
}

func NewError(kind ErrorKind, message string) Error {
	return Error{Kind: kind, Message: message}
}

func (e Error) Error() string {
	switch {
	case e.Kind == "" && e.Message == "":
		return "browser tester error"
	case e.Kind == "":
		return e.Message
	case e.Message == "":
		return string(e.Kind)
	default:
		return fmt.Sprintf("%s: %s", e.Kind, e.Message)
	}
}
