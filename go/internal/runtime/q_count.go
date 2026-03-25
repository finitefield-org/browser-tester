package runtime

func (s *Session) QCount() int {
	return s.elementCountForSelector("q")
}
