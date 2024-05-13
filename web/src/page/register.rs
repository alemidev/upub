use leptos::*;
use crate::prelude::*;

#[component]
pub fn RegisterPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>register</Breadcrumb>
			<form>
				<table class="align ma-3">
					<tr>
						<td>username</td>
						<td><input type="email" /></td>
					</tr>
					<tr>
						<td>password</td>
						<td><input type="password" /></td>
					</tr>
					<tr>
						<td colspan="2"><hr /></td>
					</tr>
					<tr>
						<td>display name</td>
						<td><input type="text" /></td>
					</tr>
					<tr>
						<td>summary</td>
						<td><input type="text" /></td>
					</tr>
					<tr>
						<td>avatar url</td>
						<td><input type="text" /></td>
					</tr>
					<tr>
						<td>banner url</td>
						<td><input type="text" /></td>
					</tr>
					<tr>
						<td colspan="2"><hr /></td>
					</tr>
					<tr>
						<td colspan="2"><input class="w-100" type="submit" value="register" /></td>
					</tr>
				</table>
			</form>
		</div>
	}
}
