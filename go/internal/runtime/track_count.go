package runtime

func (s *Session) TrackCount() int {
	return s.elementCountForSelector("track")
}
