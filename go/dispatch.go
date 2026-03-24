package browsertester

func (h *Harness) Dispatch(selector, event string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindEvent, "dispatch is unavailable")
	}
	if err := h.session.Dispatch(selector, event); err != nil {
		return NewError(ErrorKindEvent, err.Error())
	}
	return nil
}

func (h *Harness) DispatchKeyboard(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindEvent, "dispatch keyboard is unavailable")
	}
	if err := h.session.DispatchKeyboard(selector); err != nil {
		return NewError(ErrorKindEvent, err.Error())
	}
	return nil
}
