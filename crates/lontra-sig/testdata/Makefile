# Reproducible way to generate test data.
# The SSH keys are intentionally exposed and must not be used anywhere else.

ECHO = echo -n
SIGN = ssh-keygen -Y sign
SIGN_LONTRA = $(SIGN) -n lontra
TEST_MESSAGE = 'test message'

.PHONY: all
all: trusted-a@lontra.sig trusted-b@lontra.sig trusted@bad.sig untrusted@lontra.sig

trusted-a@lontra.sig: trusted_a_id_ed25519
	$(ECHO) $(TEST_MESSAGE) | $(SIGN_LONTRA) -f $< >$@

trusted-b@lontra.sig: trusted_b_id_ed25519
	$(ECHO) $(TEST_MESSAGE) | $(SIGN_LONTRA) -f $< >$@

trusted@bad.sig: trusted_a_id_ed25519
	$(ECHO) $(TEST_MESSAGE) | $(SIGN) -n 'bad' -f $< >$@

untrusted@lontra.sig: untrusted_id_ed25519
	$(ECHO) $(TEST_MESSAGE) | $(SIGN_LONTRA) -f $< >$@
