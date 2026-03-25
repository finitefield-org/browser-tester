package runtime

func (s *Session) DialogCount() int {
	return s.elementCountForSelector("dialog")
}
