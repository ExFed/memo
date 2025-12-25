;; This channel file can be passed to 'guix pull -C' or to
;; 'guix time-machine -C' to obtain the Guix revision that was
;; used to populate this profile.

(list
     ;; Note: these other commits were also used to install some of the packages in this profile:
     ;;   "9aa22e6451d465d81977981a0a90e0d21ea84815"
     ;;   "f94bca84642c33eb81023a64682eed6b9a7f9f71"
     ;;   "3bbaf5b9061aca99ba4ef570e91ad0359b3ca247"
     (channel
       (name 'guix)
       (url "https://git.guix.gnu.org/guix.git")
       (branch "master")
       (commit
         "12dcbe5876edcf574d58033f8bcb46b99bed4700")
       (introduction
         (make-channel-introduction
           "9edb3f66fd807b096b48283debdcddccfea34bad"
           (openpgp-fingerprint
             "BBB0 2DDF 2CEA F6A8 0D1D  E643 A2A0 6DF2 A33A 54FA"))))
)
