package runtime

func (s *Session) TimeCount() int {
	return s.elementCountForSelector("time")
}
