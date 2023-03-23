#include <stdbool.h>
#include <stddef.h>

#include <Foundation/Foundation.h>
#include <UIKit/UIKit.h>
#include <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

@interface FilePickerDelegate : NSObject <UIDocumentPickerDelegate> {
  void (^closure)(NSData *);
}
@property void (^closure)(NSData *);
- (instancetype)initWithClosure:(void (^)(NSData *))closure;
@end

@implementation FilePickerDelegate
@synthesize closure;

- (instancetype)initWithClosure:(void (^)(NSData *))c {
  if ([self init]) {
    self.closure = c;
  }
  return self;
}

- (void)documentPicker:(UIDocumentPickerViewController *)controller
    didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
  for (NSURL *url in urls) {
    NSData *data = [NSData dataWithContentsOfURL:url
                                         options:NSDataReadingMappedIfSafe
                                           error:nil];
    self.closure(data);
  }
  self.closure(nil);
}

- (void)documentPickerWasCancelled:
    (UIDocumentPickerViewController *)controller {
  self.closure(nil);
}
@end

FilePickerDelegate *
show_browser(UIViewController *__unsafe_unretained controller,
             const char *const *const extensions, const size_t types_len,
             const bool allow_multiple,
             void (*closure)(const void *, size_t, void *),
             void *closure_data) {
  NSMutableArray<UTType *> *types =
      [NSMutableArray arrayWithCapacity:types_len];
  for (size_t i = 0; i < types_len; i++) {
    NSString *ex = [NSString stringWithUTF8String:extensions[i]];
    UTType *type = [UTType typeWithFilenameExtension:ex];
    [types addObject:type];
  }

  UIDocumentPickerViewController *browser =
      [[UIDocumentPickerViewController alloc] initForOpeningContentTypes:types];
  browser.allowsMultipleSelection = allow_multiple ? YES : NO;
  browser.shouldShowFileExtensions = YES;

  FilePickerDelegate *delegate =
      [[FilePickerDelegate alloc] initWithClosure:^(NSData *data) {
        if (data) {
          closure([data bytes], [data length], closure_data);
        } else {
          closure(NULL, 0, closure_data);
        }
      }];
  browser.delegate = delegate;

  [controller presentViewController:browser animated:YES completion:nil];

  return delegate;
}
