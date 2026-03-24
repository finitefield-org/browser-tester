package browsertester

import (
	"errors"

	rt "browsertester/internal/runtime"
)

func (h *Harness) AssertText(selector, expected string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindAssertion, "assert text is unavailable")
	}
	if err := h.session.AssertText(selector, expected); err != nil {
		return wrapAssertionError(err)
	}
	return nil
}

func (h *Harness) AssertValue(selector, expected string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindAssertion, "assert value is unavailable")
	}
	if err := h.session.AssertValue(selector, expected); err != nil {
		return wrapAssertionError(err)
	}
	return nil
}

func (h *Harness) AssertChecked(selector string, expected bool) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindAssertion, "assert checked is unavailable")
	}
	if err := h.session.AssertChecked(selector, expected); err != nil {
		return wrapAssertionError(err)
	}
	return nil
}

func (h *Harness) AssertExists(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindAssertion, "assert exists is unavailable")
	}
	if err := h.session.AssertExists(selector); err != nil {
		return wrapAssertionError(err)
	}
	return nil
}

func wrapAssertionError(err error) error {
	if err == nil {
		return nil
	}

	var selectorErr rt.SelectorError
	if errors.As(err, &selectorErr) {
		return NewError(ErrorKindSelector, selectorErr.Error())
	}

	return NewError(ErrorKindAssertion, err.Error())
}
