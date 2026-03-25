package runtime

func (s *Session) HeaderCount() int {
	return s.elementCountForSelector("header")
}
