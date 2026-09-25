<h2><?php echo htmlspecialchars(t('contact_title')); ?></h2>
<form method="post" action="/contact.php">
<div><label><?php echo htmlspecialchars(t('contact_name')); ?></label><input type="text" name="name"></div>
<div><label><?php echo htmlspecialchars(t('contact_email')); ?></label><input type="text" name="email"></div>
<div><label><?php echo htmlspecialchars(t('contact_subject')); ?></label><input type="text" name="subject"></div>
<div><label><?php echo htmlspecialchars(t('contact_message')); ?></label><textarea name="message" rows="6" cols="40"></textarea></div>
<div><label><?php echo htmlspecialchars(t('contact_web')); ?></label><input type="text" name="webpage"></div>
<div><button type="submit"><?php echo htmlspecialchars(t('contact_send')); ?></button></div>
</form>
