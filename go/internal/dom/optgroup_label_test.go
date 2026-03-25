package dom

import "testing"

func TestOptgroupLabelForNode(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<select><optgroup id="plain" label="Group"><option>A</option></optgroup><optgroup id="legend"><legend> Legend  Name </legend><option>B</option></optgroup><optgroup id="empty"></optgroup></select>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	plainID, ok, err := store.QuerySelector("#plain")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#plain) = (%d, %v, %v), want ok", plainID, ok, err)
	}
	if got, want := store.OptgroupLabelForNode(plainID), "Group"; got != want {
		t.Fatalf("OptgroupLabelForNode(#plain) = %q, want %q", got, want)
	}

	legendID, ok, err := store.QuerySelector("#legend")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#legend) = (%d, %v, %v), want ok", legendID, ok, err)
	}
	if got, want := store.OptgroupLabelForNode(legendID), "Legend Name"; got != want {
		t.Fatalf("OptgroupLabelForNode(#legend) = %q, want %q", got, want)
	}

	emptyID, ok, err := store.QuerySelector("#empty")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#empty) = (%d, %v, %v), want ok", emptyID, ok, err)
	}
	if got := store.OptgroupLabelForNode(emptyID); got != "" {
		t.Fatalf("OptgroupLabelForNode(#empty) = %q, want empty", got)
	}

	if got := store.OptgroupLabelForNode(999); got != "" {
		t.Fatalf("OptgroupLabelForNode(invalid) = %q, want empty", got)
	}

	var nilStore *Store
	if got := nilStore.OptgroupLabelForNode(1); got != "" {
		t.Fatalf("nil OptgroupLabelForNode() = %q, want empty", got)
	}
}
