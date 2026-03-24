package browsertester

func (h *Harness) AdvanceTime(deltaMs int64) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindTimer, "advance time is unavailable")
	}
	if err := h.session.AdvanceTime(deltaMs); err != nil {
		return NewError(ErrorKindTimer, err.Error())
	}
	return nil
}
