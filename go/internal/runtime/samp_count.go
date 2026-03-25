package runtime

func (s *Session) SampCount() int {
	return s.elementCountForSelector("samp")
}
