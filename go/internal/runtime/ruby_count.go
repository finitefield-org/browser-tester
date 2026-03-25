package runtime

func (s *Session) RubyCount() int {
	return s.elementCountForSelector("ruby")
}
