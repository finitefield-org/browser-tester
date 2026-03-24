package browsertester

func (h *Harness) MatchMedia(query string) (bool, error) {
	if h == nil || h.session == nil {
		return false, NewError(ErrorKindMock, "match media is unavailable")
	}
	matches, err := h.session.MatchMedia(query)
	if err != nil {
		return false, NewError(ErrorKindMock, err.Error())
	}
	return matches, nil
}
