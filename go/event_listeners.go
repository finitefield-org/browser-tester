package browsertester

type EventListenerRegistration struct {
	NodeID int64
	Event  string
	Phase  string
	Source string
	Once   bool
}
